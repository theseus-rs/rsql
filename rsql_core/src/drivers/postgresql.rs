use crate::configuration::Configuration;
use crate::drivers::error::Result;
use crate::drivers::value::Value;
use crate::drivers::Error::UnsupportedColumnType;
use crate::drivers::{MemoryQueryResult, Results};
use async_trait::async_trait;
use bit_vec::BitVec;
use postgresql_archive::Version;
use postgresql_embedded::{PostgreSQL, Settings};
use sqlx::postgres::{PgColumn, PgConnectOptions, PgRow};
use sqlx::{Column, PgPool, Row};
use std::ops::Deref;
use std::str::FromStr;
use std::string::ToString;

const POSTGRESQL_EMBEDDED_VERSION: &str = "16.2.3";

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::drivers::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "postgresql"
    }

    async fn connect(
        &self,
        configuration: &Configuration,
        url: &str,
    ) -> Result<Box<dyn crate::drivers::Connection>> {
        let connection = Connection::new(configuration, url).await?;
        Ok(Box::new(connection))
    }
}

#[derive(Debug)]
pub(crate) struct Connection {
    postgresql: Option<PostgreSQL>,
    pool: PgPool,
}

impl Connection {
    pub(crate) async fn new(configuration: &Configuration, url: &str) -> Result<Connection> {
        let mut database_url = url.to_string();
        let postgresql = if url.starts_with("postgresql::embedded:") {
            let version = Version::from_str(POSTGRESQL_EMBEDDED_VERSION)?;
            let settings = if let Some(config_dir) = &configuration.config_dir {
                Settings {
                    installation_dir: config_dir.join("postgresql"),
                    ..Default::default()
                }
            } else {
                Settings::default()
            };

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
impl crate::drivers::Connection for Connection {
    async fn execute(&self, sql: &str) -> Result<Results> {
        let rows = sqlx::query(sql).execute(&self.pool).await?.rows_affected();
        Ok(Results::Execute(rows))
    }

    async fn query(&self, sql: &str) -> Result<Results> {
        let query_rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let columns = if let Some(row) = query_rows.first() {
            row.columns()
                .iter()
                .map(|column| column.name().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let mut rows = Vec::new();
        for row in query_rows {
            let mut row_data = Vec::new();
            for column in row.columns() {
                let value = self.convert_to_value(&row, column)?;
                row_data.push(value);
            }
            rows.push(row_data);
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Results::Query(Box::new(query_result)))
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = "SELECT table_name FROM information_schema.tables \
            WHERE table_schema = 'public' ORDER BY table_name";
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let mut tables = Vec::new();

        for row in rows {
            match row.try_get::<String, _>(0) {
                Ok(table_name) => tables.push(table_name),
                Err(error) => return Err(error.into()),
            }
        }

        Ok(tables)
    }

    async fn stop(&mut self) -> Result<()> {
        self.pool.close().await;

        if let Some(postgresql) = &self.postgresql {
            match postgresql.stop().await {
                Ok(_) => Ok(()),
                Err(error) => Err(error.into()),
            }
        } else {
            Ok(())
        }
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
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<chrono::DateTime<chrono::Utc>> = value;
            Ok(value.map(|v| Value::DateTime(v.naive_utc())))
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
                "void" => Ok(None), // pg_sleep() returns void
                _ => Err(UnsupportedColumnType {
                    column_name: column_name.to_string(),
                    column_type: type_name,
                }),
            }
        }
    }
}

// postgresql::embedded::Postgres is not functioning on Windows yet
#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod test {
    use crate::configuration::Configuration;
    use crate::drivers::{DriverManager, Results, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use serde_json::json;

    const DATABASE_URL: &str = "postgresql::embedded:";

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let drivers = DriverManager::default();
        let mut connection = drivers.connect(&configuration, DATABASE_URL).await?;
        connection.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let drivers = DriverManager::default();
        let mut connection = drivers.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE person (id INTEGER, name VARCHAR(20))")
            .await?;

        let execute_results = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        if let Results::Execute(rows) = execute_results {
            assert_eq!(rows, 1);
        }

        let results = connection.query("SELECT id, name FROM person").await?;
        if let Results::Query(query_result) = results {
            assert_eq!(query_result.columns().await, vec!["id", "name"]);
            assert_eq!(query_result.rows().await.len(), 1);
            match query_result.rows().await.get(0) {
                Some(row) => {
                    assert_eq!(row.len(), 2);

                    if let Some(Value::I32(id)) = &row[0] {
                        assert_eq!(*id, 1);
                    } else {
                        assert!(false);
                    }

                    if let Some(Value::String(name)) = &row[1] {
                        assert_eq!(name, "foo");
                    } else {
                        assert!(false);
                    }
                }
                None => assert!(false),
            }
        }

        let tables = connection.tables().await?;
        assert_eq!(tables, vec!["person"]);

        connection.stop().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> anyhow::Result<Option<Value>> {
        let configuration = Configuration::default();
        let drivers = DriverManager::default();
        let mut connection = drivers.connect(&configuration, DATABASE_URL).await?;

        let results = connection.query(sql).await?;
        let mut value: Option<Value> = None;

        if let Results::Query(query_result) = results {
            assert_eq!(query_result.columns().await.len(), 1);
            assert_eq!(query_result.rows().await.len(), 1);

            if let Some(row) = query_result.rows().await.get(0) {
                assert_eq!(row.len(), 1);

                value = row[0].clone();
            }
        }

        connection.stop().await?;
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
        let time = NaiveDateTime::parse_from_str("1983-01-01 01:23:45", "%Y-%m-%d %H:%M:%S")?;
        assert_eq!(value, Value::DateTime(time));
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
}
