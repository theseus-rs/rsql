use crate::error::Result;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, QueryResult};
use async_trait::async_trait;
use bit_vec::BitVec;
use chrono::Utc;
use indoc::indoc;
use postgresql_archive::Version;
use postgresql_embedded::{PostgreSQL, Settings, Status};
use sqlx::postgres::{PgColumn, PgConnectOptions, PgRow};
use sqlx::{Column, PgPool, Row};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use tracing::debug;
use url::Url;

const POSTGRESQL_EMBEDDED_VERSION: &str = "16.2.3";

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "postgresql"
    }

    async fn connect(
        &self,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let connection = Connection::new(url, password).await?;
        Ok(Box::new(connection))
    }
}

#[derive(Debug)]
pub(crate) struct Connection {
    postgresql: Option<PostgreSQL>,
    pool: PgPool,
}

impl Connection {
    pub(crate) async fn new(url: String, password: Option<String>) -> Result<Connection> {
        let parsed_url = Url::parse(url.as_str())?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();
        let embedded = query_parameters
            .get("embedded")
            .map(|v| v == "true")
            .unwrap_or(false);
        let mut database_url = url.to_string();

        let postgresql = if embedded {
            let default_version = POSTGRESQL_EMBEDDED_VERSION.to_string();
            let specified_version = query_parameters.get("version").unwrap_or(&default_version);
            let version = Version::from_str(specified_version)?;
            let mut settings = Settings::from_url(url)?;

            if let Some(config_dir) = query_parameters.get("installation_dir") {
                settings.installation_dir = PathBuf::from(config_dir);
            }
            if let Some(password) = password {
                settings.password = password;
            }

            debug!("Starting embedded PostgreSQL {version} server");
            let mut postgresql = PostgreSQL::new(version, settings);
            postgresql.setup().await?;
            postgresql.start().await?;

            let database_name = "embedded";
            postgresql.create_database(database_name).await?;
            let settings = postgresql.settings();
            database_url = settings.url(database_name);
            Some(postgresql)
        } else {
            None
        };

        let options = PgConnectOptions::from_str(database_url.as_str())?;
        let pool = PgPool::connect_with(options).await?;
        let connection = Connection { postgresql, pool };

        Ok(connection)
    }
}

#[async_trait]
impl crate::Connection for Connection {
    async fn execute(&self, sql: &str) -> Result<u64> {
        let rows = sqlx::query(sql).execute(&self.pool).await?.rows_affected();
        Ok(rows)
    }

    async fn indexes<'table>(&mut self, table: Option<&'table str>) -> Result<Vec<String>> {
        let mut sql = indoc! {r#"
            SELECT i.relname AS index_name
              FROM pg_class t,
                   pg_class i,
                   pg_index ix,
                   pg_attribute a,
                   information_schema.tables ist
             WHERE t.oid = ix.indrelid
               AND i.oid = ix.indexrelid
               AND a.attrelid = t.oid
               AND a.attnum = ANY(ix.indkey)
               AND t.relkind = 'r'
               AND ist.table_name = t.relname
               AND ist.table_schema = current_schema()
        "#}
        .to_string();
        if table.is_some() {
            sql = format!("{sql} AND ist.table_name = $1");
        }
        sql = format!("{sql} ORDER BY index_name").to_string();
        let query_rows = match table {
            Some(table) => {
                sqlx::query(sql.as_str())
                    .bind(table)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => sqlx::query(sql.as_str()).fetch_all(&self.pool).await?,
        };
        let mut indexes = Vec::new();

        for row in query_rows {
            if let Some(column) = row.columns().first() {
                if let Some(value) = self.convert_to_value(&row, column)? {
                    indexes.push(value.to_string());
                }
            }
        }

        Ok(indexes)
    }

    async fn query(&self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let query_rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let columns: Vec<String> = query_rows
            .first()
            .map(|row| {
                row.columns()
                    .iter()
                    .map(|column| column.name().to_string())
                    .collect()
            })
            .unwrap_or_default();

        let mut rows = Vec::new();
        for row in query_rows {
            let mut row_data = Vec::new();
            for column in row.columns() {
                let value = self.convert_to_value(&row, column)?;
                row_data.push(value);
            }
            rows.push(crate::Row::new(row_data));
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = indoc! { r#"
            SELECT table_name
              FROM information_schema.tables
             WHERE table_catalog = current_database()
               AND table_schema = 'public'
             ORDER BY table_name
        "#};
        let mut query_result = self.query(sql).await?;
        let mut tables = Vec::new();

        while let Some(row) = query_result.next().await {
            if let Some(data) = row.get(0) {
                tables.push(data.to_string());
            }
        }

        Ok(tables)
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;

        if let Some(postgresql) = &self.postgresql {
            if postgresql.status() == Status::Started {
                if let Err(error) = postgresql.stop().await {
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }
}

impl Connection {
    fn convert_to_value(&self, row: &PgRow, column: &PgColumn) -> Result<Option<Value>> {
        let column_name = column.name();

        if let Ok(value) = row.try_get(column_name) {
            let value: Option<Vec<u8>> = value;
            Ok(value.map(Value::Bytes))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<String> = value;
            Ok(value.map(Value::String))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i16> = value;
            Ok(value.map(Value::I16))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i32> = value;
            Ok(value.map(Value::I32))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i64> = value;
            Ok(value.map(Value::I64))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<f32> = value;
            Ok(value.map(Value::F32))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<f64> = value;
            Ok(value.map(Value::F64))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<rust_decimal::Decimal> = value;
            Ok(value.map(|v| Value::String(v.to_string())))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<bool> = value;
            Ok(value.map(Value::Bool))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<chrono::NaiveDate> = value;
            Ok(value.map(Value::Date))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<chrono::NaiveTime> = value;
            Ok(value.map(Value::Time))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<chrono::NaiveDateTime> = value;
            Ok(value.map(Value::DateTime))
        } else if let Ok(value) = row.try_get(column.name()) {
            let value: Option<uuid::Uuid> = value;
            Ok(value.map(Value::Uuid))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<serde_json::Value> = value;
            Ok(value.map(Value::Json))
        } else {
            let column_type = column.type_info();
            let type_name = format!("{:?}", column_type.deref());
            match type_name.to_lowercase().as_str() {
                "bit" | "varbit" => {
                    let value: Option<BitVec> = row.try_get(column_name)?;
                    Ok(value.map(|v| {
                        let bit_string: String =
                            v.iter().map(|bit| if bit { '1' } else { '0' }).collect();
                        Value::String(bit_string)
                    }))
                }
                // "interval" => Ok(None), // TODO: SELECT CAST('1 year' as interval)
                // "money" => Ok(None), // TODO: SELECT CAST(1.23 as money)
                "timestamptz" => {
                    let value: Option<chrono::DateTime<Utc>> = row.try_get(column_name)?;
                    Ok(value.map(|v| Value::DateTime(v.naive_utc())))
                }
                "void" => Ok(None), // pg_sleep() returns void
                _ => Err(UnsupportedColumnType {
                    column_name: column_name.to_string(),
                    column_type: type_name,
                }),
            }
        }
    }
}

// postgresql embedded is not functioning on Windows yet
#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod test {
    use crate::{DriverManager, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use serde_json::json;

    const DATABASE_URL: &str = "postgresql://?embedded=true";

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE person (id INTEGER, name VARCHAR(20))")
            .await?;

        let rows = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection.query("SELECT id, name FROM person").await?;
        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        match query_result.next().await {
            Some(row) => {
                assert_eq!(row.len(), 2);

                if let Some(Value::I32(id)) = row.get(0) {
                    assert_eq!(*id, 1);
                } else {
                    assert!(false);
                }

                if let Some(Value::String(name)) = row.get(1) {
                    assert_eq!(name, "foo");
                } else {
                    assert!(false);
                }
            }
            None => assert!(false),
        }
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> anyhow::Result<Option<Value>> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let mut query_result = connection.query(sql).await?;
        let mut value: Option<Value> = None;

        assert_eq!(query_result.columns().await.len(), 1);

        if let Some(row) = query_result.next().await {
            assert_eq!(row.len(), 1);

            value = row.get(0).cloned();
        }
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(value)
    }

    #[tokio::test]
    async fn test_data_type_bytes() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('1' as bytea)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::Bytes(vec![49]));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_string() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('foo' as char(3))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT CAST('foo' as varchar(5))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT CAST('foo' as text)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT CAST(B'101' as bit(3))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("101".to_string()));

        let result = test_data_type("SELECT CAST(B'10101' as bit varying(5))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("10101".to_string()));

        let result = test_data_type("SELECT CAST(1.234 as numeric)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("1.234".to_string()));

        let result = test_data_type("SELECT CAST(1.234 as decimal)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("1.234".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i16() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST(32767 as smallint)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I16(32_767));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i32() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST(2147483647 as integer)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I32(2_147_483_647));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i64() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST(2147483647 as bigint)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I64(2_147_483_647));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_bool() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST(1 as bool)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f32() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST(1.234 as real)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::F32(1.234));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f64() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST(1.234 as double precision)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::F64(1.234));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_date() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('1983-01-01' as date)").await?;
        let value = result.expect("value is None");
        let date = NaiveDate::from_ymd_opt(1983, 1, 1).expect("invalid date");
        assert_eq!(value, Value::Date(date));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_time() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('1:23:45' as time)").await?;
        let value = result.expect("value is None");
        let time = NaiveTime::from_hms_opt(1, 23, 45).expect("invalid time");
        assert_eq!(value, Value::Time(time));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_date_time() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('1983-01-01 1:23:45' as timestamp)").await?;
        let value = result.expect("value is None");
        let date_time = NaiveDateTime::parse_from_str("1983-01-01 01:23:45", "%Y-%m-%d %H:%M:%S")?;
        assert_eq!(value, Value::DateTime(date_time));

        let now = Utc::now().naive_utc();
        let result = test_data_type("SELECT now()").await?;
        let value = result.expect("value is None");
        if let Value::DateTime(value) = value {
            assert!(value > now);
        } else {
            assert!(false);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_json() -> anyhow::Result<()> {
        let result = test_data_type(r#"SELECT CAST('{"key": "value"}' as json)"#).await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::Json(json!({"key": "value"})));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_none() -> anyhow::Result<()> {
        let result = test_data_type("SELECT pg_sleep(0)").await?;
        assert_eq!(result, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_not_supported() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('<a>b</a> as xml)").await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_schema() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;

        let tables = connection.tables().await?;
        assert_eq!(tables, vec!["contacts", "users"]);

        let indexes = connection.indexes(None).await?;
        assert_eq!(indexes, vec!["contacts_pkey", "users_pkey"]);

        let indexes = connection.indexes(Some("users")).await?;
        assert_eq!(indexes, vec!["users_pkey"]);

        connection.close().await?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let docker = testcontainers::clients::Cli::default();
        let postgres_image = testcontainers::RunnableImage::from(
            testcontainers_modules::postgres::Postgres::default(),
        );
        let container = docker.run(postgres_image);
        let port = container.get_host_port_ipv4(5432);

        let database_url = format!("postgresql://postgres:postgres@localhost:{}/postgres", port);
        let driver_manager = DriverManager::default();
        let connection = driver_manager.connect(database_url.as_str()).await?;

        let mut query_result = connection.query("SELECT 'foo'::TEXT").await?;
        let row = query_result.next().await.expect("no row");
        let value = row.first().expect("no value");

        assert_eq!(*value, Value::String("foo".to_string()));

        Ok(())
    }
}
