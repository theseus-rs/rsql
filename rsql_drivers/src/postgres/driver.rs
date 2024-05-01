use crate::error::Result;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{postgresql, MemoryQueryResult, Metadata, QueryResult};
use async_trait::async_trait;
use bit_vec::BitVec;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use postgresql_archive::Version;
use postgresql_embedded::{PostgreSQL, Settings, Status};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use std::time::SystemTime;
use tokio_postgres::types::{FromSql, Type};
use tokio_postgres::{Client, Column, NoTls, Row};
use tracing::debug;
use url::Url;

const POSTGRESQL_EMBEDDED_VERSION: &str = "16.2.3";

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "postgres"
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
    client: Client,
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
        let mut database_url = url.to_string().replace("postgres://", "postgresql://");

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

        let (client, connection) = tokio_postgres::connect(database_url.as_str(), NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let connection = Connection { postgresql, client };

        Ok(connection)
    }
}

#[async_trait]
impl crate::Connection for Connection {
    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let rows = self.client.execute(sql, &[]).await?;
        Ok(rows)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        postgresql::metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let statement = self.client.prepare(sql).await?;
        let query_columns = statement.columns();
        let columns: Vec<String> = query_columns
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        let query_rows = self.client.query(sql, &[]).await?;
        let mut rows = Vec::new();
        for query_row in query_rows {
            let mut row = Vec::new();
            for (index, column) in query_columns.iter().enumerate() {
                let value = self.convert_to_value(&query_row, column, index)?;
                row.push(value);
            }
            rows.push(crate::Row::new(row));
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
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
    pub(crate) fn convert_to_value(
        &self,
        row: &Row,
        column: &Column,
        column_index: usize,
    ) -> Result<Option<Value>> {
        // https://www.postgresql.org/docs/current/datatype.html
        let column_type = column.type_();
        let value = match *column_type {
            Type::BIT | Type::VARBIT => {
                let bits_value: Option<BitVec> = row.try_get(column_index)?;
                match bits_value {
                    Some(value) => {
                        let bit_string: String = value
                            .iter()
                            .map(|bit| if bit { '1' } else { '0' })
                            .collect();
                        Some(Value::String(bit_string))
                    }
                    None => None,
                }
            }
            Type::BOOL => self.get_single(row, column_index, |v: bool| Value::Bool(v))?,
            Type::BOOL_ARRAY => self.get_array(row, column_index, |v: bool| Value::Bool(v))?,
            Type::INT2 => self.get_single(row, column_index, |v: i16| Value::I16(v))?,
            Type::INT2_ARRAY => self.get_array(row, column_index, |v: i16| Value::I16(v))?,
            Type::INT4 => self.get_single(row, column_index, |v: i32| Value::I32(v))?,
            Type::INT4_ARRAY => self.get_array(row, column_index, |v: i32| Value::I32(v))?,
            Type::INT8 => self.get_single(row, column_index, |v: i64| Value::I64(v))?,
            Type::INT8_ARRAY => self.get_array(row, column_index, |v: i64| Value::I64(v))?,
            Type::FLOAT4 => self.get_single(row, column_index, |v: f32| Value::F32(v))?,
            Type::FLOAT4_ARRAY => self.get_array(row, column_index, |v: f32| Value::F32(v))?,
            Type::FLOAT8 => self.get_single(row, column_index, |v: f64| Value::F64(v))?,
            Type::FLOAT8_ARRAY => self.get_array(row, column_index, |v: f64| Value::F64(v))?,
            Type::TEXT | Type::VARCHAR | Type::CHAR | Type::BPCHAR | Type::NAME => {
                self.get_single(row, column_index, |v: String| Value::String(v))?
            }
            Type::TEXT_ARRAY | Type::VARCHAR_ARRAY | Type::CHAR_ARRAY | Type::BPCHAR_ARRAY => {
                self.get_array(row, column_index, |v: String| Value::String(v))?
            }
            Type::JSON | Type::JSONB => {
                self.get_single(row, column_index, |v: serde_json::Value| Value::Json(v))?
            }
            Type::JSON_ARRAY | Type::JSONB_ARRAY => {
                self.get_array(row, column_index, |v: serde_json::Value| Value::Json(v))?
            }
            Type::BYTEA => {
                let byte_value: Option<&[u8]> = row.try_get(column_index)?;
                byte_value.map(|value| Value::Bytes(value.to_vec()))
            }
            Type::DATE => self.get_single(row, column_index, |v: NaiveDate| Value::Date(v))?,
            Type::TIME | Type::TIMETZ => {
                self.get_single(row, column_index, |v: NaiveTime| Value::Time(v))?
            }
            Type::TIMESTAMP => {
                self.get_single(row, column_index, |v: NaiveDateTime| Value::DateTime(v))?
            }
            Type::TIMESTAMPTZ => {
                let system_time: Option<SystemTime> = row.try_get(column_index)?;
                match system_time {
                    Some(value) => {
                        let date_time: DateTime<Utc> = value.into();
                        Some(Value::DateTime(date_time.naive_utc()))
                    }
                    None => None,
                }
            }
            Type::OID => self.get_single(row, column_index, |v: u32| Value::U32(v))?,
            Type::OID_ARRAY => self.get_array(row, column_index, |v: u32| Value::U32(v))?,
            Type::VOID => None, // pg_sleep() returns void
            _ => {
                return Err(UnsupportedColumnType {
                    column_name: column.name().to_string(),
                    column_type: column_type.name().to_string(),
                });
            }
        };

        Ok(value)
    }

    fn get_single<'a, T: FromSql<'a>>(
        &self,
        row: &'a Row,
        column_index: usize,
        to_value: impl Fn(T) -> Value,
    ) -> Result<Option<Value>> {
        let value = row.try_get::<_, Option<T>>(column_index)?.map(to_value);
        Ok(value)
    }

    fn get_array<'a, T: FromSql<'a>>(
        &self,
        row: &'a Row,
        column_index: usize,
        to_value: impl Fn(T) -> Value,
    ) -> Result<Option<Value>> {
        let original_value_array = row.try_get::<_, Option<Vec<T>>>(column_index)?;
        let result = match original_value_array {
            Some(value_array) => {
                let mut values = vec![];
                for value in value_array {
                    values.push(to_value(value));
                }
                Some(Value::Array(values))
            }
            None => None,
        };
        Ok(result)
    }
}

// postgresql embedded is not functioning on Windows yet
#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod test {
    use crate::{DriverManager, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use serde_json::json;
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    use testcontainers::runners::AsyncRunner;

    const DATABASE_URL: &str = "postgres://?embedded=true";

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

        let result = test_data_type("SELECT CAST(B'10101' as bit varying(5))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("10101".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i16() -> anyhow::Result<()> {
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
    async fn test_data_type_i32() -> anyhow::Result<()> {
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
    async fn test_data_type_i64() -> anyhow::Result<()> {
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
    async fn test_data_type_bool() -> anyhow::Result<()> {
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
    async fn test_data_type_f32() -> anyhow::Result<()> {
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
    async fn test_data_type_f64() -> anyhow::Result<()> {
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

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let postgres_image = testcontainers::RunnableImage::from(
            testcontainers_modules::postgres::Postgres::default(),
        );
        let container = postgres_image.start().await;
        let port = container.get_host_port_ipv4(5432).await;

        let database_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        let mut query_result = connection.query("SELECT 'foo'::TEXT").await?;
        let row = query_result.next().await.expect("no row");
        let value = row.first().expect("no value");

        assert_eq!(*value, Value::String("foo".to_string()));

        Ok(())
    }
}
