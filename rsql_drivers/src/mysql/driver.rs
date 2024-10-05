use crate::error::Result;
use crate::mysql::metadata;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, Metadata, QueryResult};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlparser::dialect::{Dialect, MySqlDialect};
use sqlx::mysql::{MySqlColumn, MySqlConnectOptions, MySqlRow};
use sqlx::types::time::OffsetDateTime;
use sqlx::{Column, MySqlPool, Row};
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "mysql"
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
    pool: MySqlPool,
}

impl Connection {
    pub(crate) async fn new(url: String, _password: Option<String>) -> Result<Connection> {
        let options = MySqlConnectOptions::from_str(url.as_str())?;
        let pool = MySqlPool::connect_with(options).await?;
        let connection = Connection { pool };

        Ok(connection)
    }
}

#[async_trait]
impl crate::Connection for Connection {
    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let rows = sqlx::query(sql).execute(&self.pool).await?.rows_affected();
        Ok(rows)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
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
                let value = Self::convert_to_value(&row, column)?;
                row_data.push(value);
            }
            rows.push(row_data);
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(MySqlDialect {})
    }
}

impl Connection {
    fn convert_to_value(row: &MySqlRow, column: &MySqlColumn) -> Result<Value> {
        let column_name = column.name();

        if let Ok(value) = row.try_get::<Option<String>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::String(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<Vec<u8>>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Bytes(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i16>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I16(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i32>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I32(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i64>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I64(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<f32>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::F32(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<f64>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::F64(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<rust_decimal::Decimal>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::String(v.to_string())),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<bool>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Bool(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<NaiveDate>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Date(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<NaiveTime>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Time(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<NaiveDateTime>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::DateTime(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<OffsetDateTime>, &str>(column_name) {
            match value {
                Some(v) => {
                    let date = v.date();
                    let time = v.time();
                    let date_time_string = format!("{date} {time}");
                    let date_time =
                        NaiveDateTime::parse_from_str(&date_time_string, "%Y-%m-%d %H:%M:%S%.f")
                            .expect("invalid date");
                    Ok(Value::DateTime(date_time))
                }
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<serde_json::Value>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Json(v)),
                None => Ok(Value::Null),
            }
        } else {
            let column_type = column.type_info();
            let type_name = format!("{column_type:?}");
            return Err(UnsupportedColumnType {
                column_name: column_name.to_string(),
                column_type: type_name,
            });
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Connection, DriverManager, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use indoc::indoc;
    use serde_json::json;
    use testcontainers::runners::AsyncRunner;

    #[allow(dead_code)]
    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        // Skip tests on GitHub Actions for non-Linux platforms; the test containers fail to run.
        if std::env::var("GITHUB_ACTIONS").is_ok() && !cfg!(target_os = "linux") {
            return Ok(());
        }

        let mysql_image =
            testcontainers::ContainerRequest::from(testcontainers_modules::mysql::Mysql::default());
        let container = mysql_image.start().await?;
        let port = container.get_host_port_ipv4(3306).await?;

        let database_url = &format!("mysql://root@127.0.0.1:{port}/mysql");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        test_connection_interface(&mut *connection).await?;
        test_data_types(&mut *connection).await?;

        container.stop().await?;
        container.rm().await?;
        Ok(())
    }

    async fn test_connection_interface(connection: &mut dyn Connection) -> anyhow::Result<()> {
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
            Some(vec![Value::I16(1), Value::String("foo".to_string())])
        );
        assert!(query_result.next().await.is_none());

        let db_metadata = connection.metadata().await?;
        let schema = db_metadata
            .current_schema()
            .expect("expected at least one schema");
        assert!(schema.tables().iter().any(|table| table.name() == "person"));

        Ok(())
    }

    async fn test_data_types(connection: &mut dyn Connection) -> anyhow::Result<()> {
        let sql = indoc! {r"
            CREATE TABLE data_types (
                char_type CHAR,
                varchar_type VARCHAR(50),
                text_type TEXT,
                binary_type BINARY(3),
                varbinary_type VARBINARY(50),
                blob_type BLOB,
                tinyint_type TINYINT,
                smallint_type SMALLINT,
                mediumint_type MEDIUMINT,
                int_type INT,
                bigint_type BIGINT,
                decimal_type DECIMAL(5,2),
                float_type FLOAT,
                double_type DOUBLE,
                date_type DATE,
                time_type TIME,
                datetime_type DATETIME,
                timestamp_type TIMESTAMP,
                json_type JSON
            )
        "};
        let _ = connection.execute(sql).await?;

        let sql = indoc! {r#"
            INSERT INTO data_types (
                char_type, varchar_type, text_type, binary_type, varbinary_type, blob_type,
                tinyint_type, smallint_type, mediumint_type, int_type, bigint_type,
                decimal_type, float_type, double_type, date_type, time_type, datetime_type,
                timestamp_type, json_type
            ) VALUES (
                 'a', 'foo', 'foo', 'foo', 'foo', 'foo',
                 127, 32767, 8388607, 2147483647, 9223372036854775807,
                 123.45, 123.0, 123.0, '2022-01-01', '14:30:00', '2022-01-01 14:30:00',
                 '2022-01-01 14:30:00', '{"key": "value"}'
             )
        "#};
        let _ = connection.execute(sql).await?;

        let sql = indoc! {r"
            SELECT char_type, varchar_type, text_type, binary_type, varbinary_type, blob_type,
                   tinyint_type, smallint_type, mediumint_type, int_type, bigint_type,
                   decimal_type, float_type, double_type, date_type, time_type, datetime_type,
                   timestamp_type, json_type
              FROM data_types
        "};
        let mut query_result = connection.query(sql).await?;
        assert_eq!(
            query_result.next().await,
            Some(vec![
                Value::String("a".to_string()),
                Value::String("foo".to_string()),
                Value::String("foo".to_string()),
                Value::Bytes("foo".as_bytes().to_vec()),
                Value::Bytes("foo".as_bytes().to_vec()),
                Value::Bytes("foo".as_bytes().to_vec()),
                Value::I16(127),
                Value::I16(32_767),
                Value::I32(8_388_607),
                Value::I32(2_147_483_647),
                Value::I64(9_223_372_036_854_775_807),
                Value::String("123.45".to_string()),
                Value::F32(123.0),
                Value::F32(123.0),
                Value::Date(NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid date")),
                Value::Time(NaiveTime::from_hms_opt(14, 30, 00).expect("invalid time")),
                Value::DateTime(NaiveDateTime::parse_from_str(
                    "2022-01-01 14:30:00",
                    "%Y-%m-%d %H:%M:%S"
                )?),
                Value::DateTime(NaiveDateTime::parse_from_str(
                    "2022-01-01 14:30:00",
                    "%Y-%m-%d %H:%M:%S"
                )?),
                Value::Json(json!({"key": "value"}))
            ])
        );
        assert!(query_result.next().await.is_none());

        Ok(())
    }
}
