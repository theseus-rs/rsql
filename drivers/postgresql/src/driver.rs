use crate::metadata;
use crate::results::PostgreSqlQueryResult;
use async_trait::async_trait;
use file_type::FileType;
use postgresql_embedded::{PostgreSQL, Settings, Status, VersionReq};
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{Metadata, QueryResult, Result, StatementMetadata};
use rsql_driver::{ToSql, Value, convert_to_numbered_placeholders};
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, PostgreSqlDialect};
use sqlx::postgres::{PgArguments, PgConnectOptions};
use sqlx::{Column, PgPool, Postgres, Row};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use tracing::debug;
use url::Url;

const POSTGRESQL_EMBEDDED_VERSION: &str = "=18.2.0";

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "postgresql"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url).map_err(|error| InvalidUrl(error.to_string()))?;
        let password = parsed_url.password().map(ToString::to_string);
        let connection = Connection::new(url, password).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Connection {
    url: String,
    postgresql: Option<PostgreSQL>,
    pool: PgPool,
}

impl Connection {
    /// Creates a new connection to the `PostgreSQL` database.
    ///
    /// # Errors
    /// if the connection to the database fails.
    pub async fn new(url: &str, password: Option<String>) -> Result<Connection> {
        let parsed_url = Url::parse(url)?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();
        let embedded = query_parameters
            .get("embedded")
            .is_some_and(|value| value == "true");
        let mut database_url = url.to_string();

        let postgresql = if embedded {
            let mut settings =
                Settings::from_url(url).map_err(|error| IoError(error.to_string()))?;

            if !query_parameters.contains_key("version") {
                let version = VersionReq::from_str(POSTGRESQL_EMBEDDED_VERSION)
                    .map_err(|error| IoError(error.to_string()))?;
                settings.version = version;
            }
            if let Some(config_dir) = query_parameters.get("installation_dir") {
                settings.installation_dir = PathBuf::from(config_dir);
            }
            if let Some(password) = password {
                settings.password = password;
            }

            let mut postgresql = PostgreSQL::new(settings);
            postgresql
                .setup()
                .await
                .map_err(|error| IoError(error.to_string()))?;
            let version = postgresql.settings().version.clone();
            debug!("Starting embedded PostgreSQL {version} server");
            postgresql
                .start()
                .await
                .map_err(|error| IoError(error.to_string()))?;

            let database_name = "embedded";
            postgresql
                .create_database(database_name)
                .await
                .map_err(|error| IoError(error.to_string()))?;
            let settings = postgresql.settings();
            database_url = settings.url(database_name);
            Some(postgresql)
        } else {
            None
        };

        let options = PgConnectOptions::from_str(database_url.as_str())
            .map_err(|error| IoError(error.to_string()))?;
        let pool = PgPool::connect_with(options)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let connection = Connection {
            url: url.to_string(),
            postgresql,
            pool,
        };

        Ok(connection)
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let sql = convert_to_numbered_placeholders(sql);
        let values = rsql_driver::to_values(params);
        let mut query = sqlx::query(&sql);
        for value in &values {
            query = bind_pg_value(query, value);
        }
        let rows = query
            .execute(&self.pool)
            .await
            .map_err(|error| IoError(error.to_string()))?
            .rows_affected();
        Ok(rows)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let sql = convert_to_numbered_placeholders(sql);
        let values = rsql_driver::to_values(params);
        let mut query = sqlx::query(&sql);
        for value in &values {
            query = bind_pg_value(query, value);
        }
        let query_rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let columns: Vec<String> = query_rows
            .first()
            .map(|row| {
                row.columns()
                    .iter()
                    .map(|column| column.name().to_string())
                    .collect()
            })
            .unwrap_or_default();

        let query_result = PostgreSqlQueryResult::new(columns, query_rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;

        if let Some(postgresql) = &self.postgresql
            && postgresql.status() == Status::Started
            && let Err(error) = postgresql.stop().await
        {
            return Err(IoError(error.to_string()));
        }

        Ok(())
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(PostgreSqlDialect {})
    }

    fn match_statement(&self, statement: &Statement) -> StatementMetadata {
        let default = self.default_match_statement(statement);
        match default {
            StatementMetadata::Unknown => match statement {
                Statement::CreateExtension { .. } | Statement::CreateFunction { .. } => {
                    StatementMetadata::DDL
                }
                _ => StatementMetadata::Unknown,
            },
            other => other,
        }
    }
}

fn bind_pg_value<'q>(
    query: sqlx::query::Query<'q, Postgres, PgArguments>,
    value: &'q Value,
) -> sqlx::query::Query<'q, Postgres, PgArguments> {
    match value {
        Value::Null => query.bind(None::<String>),
        Value::Bool(v) => query.bind(*v),
        Value::I8(v) => query.bind(i16::from(*v)),
        Value::I16(v) => query.bind(*v),
        Value::I32(v) => query.bind(*v),
        Value::I64(v) => query.bind(*v),
        Value::U8(v) => query.bind(i16::from(*v)),
        Value::U16(v) => query.bind(i32::from(*v)),
        Value::U32(v) => query.bind(i64::from(*v)),
        Value::U64(v) => query.bind(*v as i64),
        Value::F32(v) => query.bind(*v),
        Value::F64(v) => query.bind(*v),
        Value::String(v) => query.bind(v.as_str()),
        Value::Bytes(v) => query.bind(v.as_slice()),
        Value::Decimal(v) => query.bind(*v),
        _ => query.bind(value.to_string()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use jiff::tz::Offset;
    use jiff::{Timestamp, civil};
    use rsql_driver::Driver;
    use serde_json::json;

    const DATABASE_URL: &str = "postgresql://?embedded=true";

    #[tokio::test]
    async fn test_driver_connect() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;
        assert_eq!(DATABASE_URL, connection.url());
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE person (id INTEGER, name VARCHAR(20))", &[])
            .await?;

        let rows = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')", &[])
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection.query("SELECT id, name FROM person", &[]).await?;
        assert_eq!(query_result.columns(), vec!["id", "name"]);
        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![Value::I32(1), Value::String("foo".to_string())])
        );
        assert!(query_result.next().await.is_none());

        let metadata = connection.metadata().await?;
        let catalog = metadata.current_catalog().expect("catalog");
        let schema = catalog.current_schema().expect("schema");
        assert!(schema.tables().iter().any(|table| table.name() == "person"));

        connection.close().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> Result<Option<Value>> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let mut query_result = connection.query(sql, &[]).await?;
        let mut value: Option<Value> = None;

        assert_eq!(query_result.columns().len(), 1);

        if let Some(row) = query_result.next().await {
            assert_eq!(row.len(), 1);

            value = row.first().cloned();
        }
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(value)
    }

    #[tokio::test]
    async fn test_data_type_bytes() -> Result<()> {
        let result = test_data_type("SELECT CAST('1' as bytea)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::Bytes(vec![49]));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_string() -> Result<()> {
        let result = test_data_type("SELECT CAST('foo' as char(3))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT CAST('foo' as varchar(5))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT 'foo'::TEXT").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT ARRAY['foo','bar']::TEXT[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::String("foo".to_string()));
            assert_eq!(value[1], Value::String("bar".to_string()));
        }

        let result = test_data_type("SELECT CAST(B'101' as bit(3))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("101".to_string()));

        let result =
            test_data_type("SELECT ARRAY[CAST(B'10' as bit(2)), CAST(B'101' as bit(3))]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::String("10".to_string()));
            assert_eq!(value[1], Value::String("101".to_string()));
        }

        let result = test_data_type("SELECT CAST(B'10101' as bit varying(5))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("10101".to_string()));

        let result = test_data_type(
            "SELECT ARRAY[CAST(B'10' as bit varying(5)), CAST(B'101' as bit varying(5))]",
        )
        .await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::String("10".to_string()));
            assert_eq!(value[1], Value::String("101".to_string()));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i16() -> Result<()> {
        let result = test_data_type("SELECT 32767::INT2").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I16(32_767));

        let result = test_data_type("SELECT ARRAY[0,32767]::INT2[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::I16(0));
            assert_eq!(value[1], Value::I16(32_767));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i32() -> Result<()> {
        let result = test_data_type("SELECT 2147483647::INT4").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I32(2_147_483_647));

        let result = test_data_type("SELECT ARRAY[0,2147483647]::INT4[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::I32(0));
            assert_eq!(value[1], Value::I32(2_147_483_647));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i64() -> Result<()> {
        let result = test_data_type("SELECT 2147483647::INT8").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I64(2_147_483_647));

        let result = test_data_type("SELECT ARRAY[0,2147483647]::INT8[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::I64(0));
            assert_eq!(value[1], Value::I64(2_147_483_647));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_bool() -> Result<()> {
        let result = test_data_type("SELECT 1::BOOL").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::Bool(true));

        let result = test_data_type("SELECT ARRAY[0,1]::BOOL[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::Bool(false));
            assert_eq!(value[1], Value::Bool(true));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f32() -> Result<()> {
        let result = test_data_type("SELECT 1.234::FLOAT4").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::F32(1.234));

        let result = test_data_type("SELECT ARRAY[0,1.234]::FLOAT4[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::F32(0.0));
            assert_eq!(value[1], Value::F32(1.234));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f64() -> Result<()> {
        let result = test_data_type("SELECT 1.234::FLOAT8").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::F64(1.234));

        let result = test_data_type("SELECT ARRAY[0,1.234]::FLOAT8[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::F64(0.0));
            assert_eq!(value[1], Value::F64(1.234));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_decimal() -> Result<()> {
        let result = test_data_type("SELECT 1.23::DECIMAL(10,2)").await?;
        let value = result.expect("value is None");
        assert_eq!(
            value,
            Value::Decimal(rust_decimal::Decimal::from_str("1.23").expect("invalid decimal"))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_date() -> Result<()> {
        let result = test_data_type("SELECT CAST('1983-01-01' as date)").await?;
        let value = result.expect("value is None");
        let date = civil::date(1983, 1, 1);
        assert_eq!(value, Value::Date(date));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_time() -> Result<()> {
        let result = test_data_type("SELECT CAST('1:23:45' as time)").await?;
        let value = result.expect("value is None");
        let time = civil::time(1, 23, 45, 0);
        assert_eq!(value, Value::Time(time));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_date_time() -> Result<()> {
        let result = test_data_type("SELECT CAST('1983-01-01 1:23:45' as timestamp)").await?;
        let value = result.expect("value is None");
        let date_time = civil::datetime(1983, 1, 1, 1, 23, 45, 0);
        assert_eq!(value, Value::DateTime(date_time));

        let now = Offset::UTC.to_datetime(Timestamp::now());
        let result = test_data_type("SELECT now()").await?;
        let value = result.expect("value is None");
        if let Value::DateTime(value) = value {
            assert!(value > now);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_json() -> Result<()> {
        let result = test_data_type(r#"SELECT CAST('{"key": "value"}' as json)"#).await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::from(json!({"key": "value"})));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_null() -> Result<()> {
        let result = test_data_type("SELECT pg_sleep(0)").await?;
        assert_eq!(result, Some(Value::Null));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_not_supported() -> Result<()> {
        let result = test_data_type("SELECT CAST('<a>b</a> as xml)").await;
        assert!(result.is_err());
        Ok(())
    }
}
