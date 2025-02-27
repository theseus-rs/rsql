use async_trait::async_trait;
use bit_vec::BitVec;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use file_type::FileType;
use postgresql_embedded::{PostgreSQL, Settings, Status, VersionReq};
use rsql_driver::Error::{InvalidUrl, IoError, UnsupportedColumnType};
use rsql_driver::{MemoryQueryResult, Metadata, QueryResult, Result, StatementMetadata, Value};
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, PostgreSqlDialect};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use std::time::SystemTime;
use tokio_postgres::types::{FromSql, Type};
use tokio_postgres::{Client, Column, NoTls, Row};
use tracing::debug;
use url::Url;
use uuid::Uuid;

const POSTGRESQL_EMBEDDED_VERSION: &str = "=17.3.0";

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "postgres"
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
    client: Client,
}

impl Connection {
    pub(crate) async fn new(url: &str, password: Option<String>) -> Result<Connection> {
        let parsed_url = Url::parse(url)?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();
        let embedded = query_parameters
            .get("embedded")
            .is_some_and(|value| value == "true");
        let mut database_url = url.to_string().replace("postgres://", "postgresql://");

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

        let (client, connection) = tokio_postgres::connect(database_url.as_str(), NoTls)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {e}");
            }
        });
        let connection = Connection {
            url: url.to_string(),
            postgresql,
            client,
        };

        Ok(connection)
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let rows = self
            .client
            .execute(sql, &[])
            .await
            .map_err(|error| IoError(error.to_string()))?;
        Ok(rows)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        rsql_driver_postgresql::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let statement = self
            .client
            .prepare(sql)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let query_columns = statement.columns();
        let columns: Vec<String> = query_columns
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        let query_rows = self
            .client
            .query(sql, &[])
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let mut rows = Vec::new();
        for query_row in query_rows {
            let mut row = Vec::new();
            for (index, column) in query_columns.iter().enumerate() {
                let value = Self::convert_to_value(&query_row, column, index)?;
                row.push(value);
            }
            rows.push(row);
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(postgresql) = &self.postgresql {
            if postgresql.status() == Status::Started {
                if let Err(error) = postgresql.stop().await {
                    return Err(IoError(error.to_string()));
                }
            }
        }

        Ok(())
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

impl Connection {
    pub(crate) fn convert_to_value(
        row: &Row,
        column: &Column,
        column_index: usize,
    ) -> Result<Value> {
        // https://www.postgresql.org/docs/current/datatype.html
        let column_type = column.type_();
        let value = match *column_type {
            Type::BIT | Type::VARBIT => Self::get_single(row, column_index, |v: BitVec| {
                Value::String(Self::bit_string(&v))
            })?,
            Type::BIT_ARRAY | Type::VARBIT_ARRAY => {
                Self::get_array(row, column_index, |v: BitVec| {
                    Value::String(Self::bit_string(&v))
                })?
            }
            Type::BOOL => Self::get_single(row, column_index, |v: bool| Value::Bool(v))?,
            Type::BOOL_ARRAY => Self::get_array(row, column_index, |v: bool| Value::Bool(v))?,
            Type::INT2 => Self::get_single(row, column_index, |v: i16| Value::I16(v))?,
            Type::INT2_ARRAY => Self::get_array(row, column_index, |v: i16| Value::I16(v))?,
            Type::INT4 => Self::get_single(row, column_index, |v: i32| Value::I32(v))?,
            Type::INT4_ARRAY => Self::get_array(row, column_index, |v: i32| Value::I32(v))?,
            Type::INT8 => Self::get_single(row, column_index, |v: i64| Value::I64(v))?,
            Type::INT8_ARRAY => Self::get_array(row, column_index, |v: i64| Value::I64(v))?,
            Type::FLOAT4 => Self::get_single(row, column_index, |v: f32| Value::F32(v))?,
            Type::FLOAT4_ARRAY => Self::get_array(row, column_index, |v: f32| Value::F32(v))?,
            Type::FLOAT8 => Self::get_single(row, column_index, |v: f64| Value::F64(v))?,
            Type::FLOAT8_ARRAY => Self::get_array(row, column_index, |v: f64| Value::F64(v))?,
            Type::TEXT | Type::VARCHAR | Type::CHAR | Type::BPCHAR | Type::NAME => {
                Self::get_single(row, column_index, |v: String| Value::String(v))?
            }
            Type::TEXT_ARRAY | Type::VARCHAR_ARRAY | Type::CHAR_ARRAY | Type::BPCHAR_ARRAY => {
                Self::get_array(row, column_index, |v: String| Value::String(v))?
            }
            Type::NUMERIC => Self::get_single(row, column_index, |v: rust_decimal::Decimal| {
                Value::Decimal(v)
            })?,
            Type::NUMERIC_ARRAY => {
                Self::get_single(row, column_index, |v: rust_decimal::Decimal| {
                    Value::Decimal(v)
                })?
            }
            Type::JSON | Type::JSONB => {
                Self::get_single(row, column_index, |v: serde_json::Value| Value::from(v))?
            }
            Type::JSON_ARRAY | Type::JSONB_ARRAY => {
                Self::get_array(row, column_index, |v: serde_json::Value| Value::from(v))?
            }
            Type::BYTEA => {
                let byte_value: Option<&[u8]> = row
                    .try_get(column_index)
                    .map_err(|error| IoError(error.to_string()))?;
                match byte_value {
                    Some(value) => Value::Bytes(value.to_vec()),
                    None => Value::Null,
                }
            }
            Type::DATE => Self::get_single(row, column_index, |v: NaiveDate| Value::Date(v))?,
            Type::TIME | Type::TIMETZ => {
                Self::get_single(row, column_index, |v: NaiveTime| Value::Time(v))?
            }
            Type::TIMESTAMP => {
                Self::get_single(row, column_index, |v: NaiveDateTime| Value::DateTime(v))?
            }
            Type::TIMESTAMPTZ => {
                let system_time: Option<SystemTime> = row
                    .try_get(column_index)
                    .map_err(|error| IoError(error.to_string()))?;
                match system_time {
                    Some(value) => {
                        let date_time: DateTime<Utc> = value.into();
                        Value::DateTime(date_time.naive_utc())
                    }
                    None => Value::Null,
                }
            }
            Type::OID => Self::get_single(row, column_index, |v: u32| Value::U32(v))?,
            Type::OID_ARRAY => Self::get_array(row, column_index, |v: u32| Value::U32(v))?,
            Type::UUID => Self::get_single(row, column_index, |v: Uuid| Value::Uuid(v))?,
            Type::UUID_ARRAY => Self::get_array(row, column_index, |v: Uuid| Value::Uuid(v))?,
            Type::VOID => Value::Null, // pg_sleep() returns void
            _ => {
                return Err(UnsupportedColumnType {
                    column_name: column.name().to_string(),
                    column_type: column_type.name().to_string(),
                });
            }
        };

        Ok(value)
    }

    fn get_single<'r, T: FromSql<'r>>(
        row: &'r Row,
        column_index: usize,
        to_value: impl Fn(T) -> Value,
    ) -> Result<Value> {
        match row
            .try_get::<_, Option<T>>(column_index)
            .map_err(|error| IoError(error.to_string()))?
            .map(to_value)
        {
            Some(value) => Ok(value),
            None => Ok(Value::Null),
        }
    }

    fn get_array<'r, T: FromSql<'r>>(
        row: &'r Row,
        column_index: usize,
        to_value: impl Fn(T) -> Value,
    ) -> Result<Value> {
        let original_value_array = row
            .try_get::<_, Option<Vec<T>>>(column_index)
            .map_err(|error| IoError(error.to_string()))?;
        let result = match original_value_array {
            Some(value_array) => {
                let mut values = vec![];
                for value in value_array {
                    values.push(to_value(value));
                }
                Value::Array(values)
            }
            None => Value::Null,
        };
        Ok(result)
    }

    fn bit_string(value: &BitVec) -> String {
        let bit_string: String = value
            .iter()
            .map(|bit| if bit { '1' } else { '0' })
            .collect();
        bit_string
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use rsql_driver::{Driver, Value};
    use serde_json::json;

    const DATABASE_URL: &str = "postgres://?embedded=true";

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
            .execute("CREATE TABLE person (id INTEGER, name VARCHAR(20))")
            .await?;

        let rows = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection.query("SELECT id, name FROM person").await?;
        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I32(1), Value::String("foo".to_string())])
        );
        assert!(query_result.next().await.is_none());

        let db_metadata = connection.metadata().await?;
        let schema = db_metadata
            .current_schema()
            .expect("expected at least one schema");
        assert!(schema.tables().iter().any(|table| table.name() == "person"));

        connection.close().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> Result<Option<Value>> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let mut query_result = connection.query(sql).await?;
        let mut value: Option<Value> = None;

        assert_eq!(query_result.columns().await.len(), 1);

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
        let date = NaiveDate::from_ymd_opt(1983, 1, 1).expect("invalid date");
        assert_eq!(value, Value::Date(date));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_time() -> Result<()> {
        let result = test_data_type("SELECT CAST('1:23:45' as time)").await?;
        let value = result.expect("value is None");
        let time = NaiveTime::from_hms_opt(1, 23, 45).expect("invalid time");
        assert_eq!(value, Value::Time(time));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_date_time() -> Result<()> {
        let result = test_data_type("SELECT CAST('1983-01-01 1:23:45' as timestamp)").await?;
        let value = result.expect("value is None");
        let date_time = NaiveDateTime::parse_from_str("1983-01-01 01:23:45", "%Y-%m-%d %H:%M:%S")
            .map_err(|error| IoError(error.to_string()))?;
        assert_eq!(value, Value::DateTime(date_time));

        let now = Utc::now().naive_utc();
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
