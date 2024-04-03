use crate::error::Result;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, QueryResult};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use indoc::indoc;
use sqlx::mysql::{MySqlColumn, MySqlConnectOptions, MySqlRow};
use sqlx::types::time;
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
    async fn execute(&self, sql: &str) -> Result<u64> {
        let rows = sqlx::query(sql).execute(&self.pool).await?.rows_affected();
        Ok(rows)
    }

    async fn indexes<'table>(&mut self, table: Option<&'table str>) -> Result<Vec<String>> {
        let mut sql = indoc! {r#"
            SELECT DISTINCT index_name
              FROM information_schema.statistics
             WHERE table_schema = DATABASE()
        "#}
        .to_string();
        if table.is_some() {
            sql = format!("{sql} AND table_name = ?");
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

    async fn query(&self, sql: &str, limit: u64) -> Result<Box<dyn QueryResult>> {
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
            rows.push(row_data);

            if limit > 0 && rows.len() >= limit as usize {
                break;
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = indoc! { r#"
            SELECT table_name
              FROM information_schema.tables
             WHERE table_schema = DATABASE()
             ORDER BY table_name
        "#};
        let query_result = self.query(sql, 0).await?;
        let mut tables = Vec::new();

        for row in query_result.rows().await {
            if let Some(data) = &row[0] {
                tables.push(data.to_string());
            }
        }

        Ok(tables)
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

impl Connection {
    fn convert_to_value(&self, row: &MySqlRow, column: &MySqlColumn) -> Result<Option<Value>> {
        let column_name = column.name();

        if let Ok(value) = row.try_get(column_name) {
            let value: Option<String> = value;
            Ok(value.map(Value::String))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<Vec<u8>> = value;
            Ok(value.map(Value::Bytes))
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
            let value: Option<time::OffsetDateTime> = value;
            let date_time = value.map(|v| {
                let date = v.date();
                let time = v.time();
                let date_time_string = format!("{} {}", date, time);
                NaiveDateTime::parse_from_str(&date_time_string, "%Y-%m-%d %H:%M:%S%.f")
                    .expect("invalid date")
            });
            Ok(date_time.map(Value::DateTime))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<serde_json::Value> = value;
            Ok(value.map(Value::Json))
        } else {
            let column_type = column.type_info();
            let type_name = format!("{:?}", column_type);
            return Err(UnsupportedColumnType {
                column_name: column_name.to_string(),
                column_type: type_name,
            });
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[cfg(test)]
mod test {
    use crate::{Connection, DriverManager, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use indoc::indoc;
    use serde_json::json;

    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let docker = testcontainers::clients::Cli::default();
        let mysql_image =
            testcontainers::RunnableImage::from(testcontainers_modules::mysql::Mysql::default());
        let container = docker.run(mysql_image);
        let port = container.get_host_port_ipv4(3306);

        let database_url = &format!("mysql://root@127.0.0.1:{port}/mysql");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        test_limit_rows(&*connection).await?;
        test_connection_interface(&*connection).await?;
        test_data_types(&*connection).await?;
        test_schema(&mut *connection).await?;

        Ok(())
    }

    async fn test_limit_rows(connection: &dyn Connection) -> anyhow::Result<()> {
        let query_result = connection.query("SELECT 1 UNION ALL SELECT 2", 1).await?;
        assert_eq!(query_result.rows().await.len(), 1);
        Ok(())
    }

    async fn test_connection_interface(connection: &dyn Connection) -> anyhow::Result<()> {
        let _ = connection
            .execute("CREATE TABLE person (id INTEGER, name VARCHAR(20))")
            .await?;

        let rows = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        assert_eq!(rows, 1);

        let query_result = connection.query("SELECT id, name FROM person", 0).await?;
        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(query_result.rows().await.len(), 1);
        match query_result.rows().await.get(0) {
            Some(row) => {
                assert_eq!(row.len(), 2);

                if let Some(Value::I16(id)) = &row[0] {
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
        Ok(())
    }

    async fn test_data_types(connection: &dyn Connection) -> anyhow::Result<()> {
        let sql = indoc! {r#"
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
        "#};
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

        let sql = indoc! {r#"
            SELECT char_type, varchar_type, text_type, binary_type, varbinary_type, blob_type,
                   tinyint_type, smallint_type, mediumint_type, int_type, bigint_type,
                   decimal_type, float_type, double_type, date_type, time_type, datetime_type,
                   timestamp_type, json_type
              FROM data_types
        "#};
        let query_result = connection.query(sql, 0).await?;

        if let Some(row) = query_result.rows().await.first() {
            assert_eq!(row[0].clone().unwrap(), Value::String("a".to_string()));
            assert_eq!(row[1].clone().unwrap(), Value::String("foo".to_string()));
            assert_eq!(row[2].clone().unwrap(), Value::String("foo".to_string()));
            assert_eq!(
                row[3].clone().unwrap(),
                Value::Bytes("foo".as_bytes().to_vec())
            );
            assert_eq!(
                row[4].clone().unwrap(),
                Value::Bytes("foo".as_bytes().to_vec())
            );
            assert_eq!(
                row[5].clone().unwrap(),
                Value::Bytes("foo".as_bytes().to_vec())
            );
            assert_eq!(row[6].clone().unwrap(), Value::I16(127));
            assert_eq!(row[7].clone().unwrap(), Value::I16(32_767));
            assert_eq!(row[8].clone().unwrap(), Value::I32(8_388_607));
            assert_eq!(row[9].clone().unwrap(), Value::I32(2_147_483_647));
            assert_eq!(
                row[10].clone().unwrap(),
                Value::I64(9_223_372_036_854_775_807)
            );
            assert_eq!(
                row[11].clone().unwrap(),
                Value::String("123.45".to_string())
            );
            assert_eq!(row[12].clone().unwrap(), Value::F32(123.0));
            assert_eq!(row[13].clone().unwrap(), Value::F32(123.0));
            let date = NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid date");
            assert_eq!(row[14].clone().unwrap(), Value::Date(date));
            let time = NaiveTime::from_hms_opt(14, 30, 00).expect("invalid time");
            assert_eq!(row[15].clone().unwrap(), Value::Time(time));
            let date_time =
                NaiveDateTime::parse_from_str("2022-01-01 14:30:00", "%Y-%m-%d %H:%M:%S")?;
            assert_eq!(row[16].clone().unwrap(), Value::DateTime(date_time));
            // assert_eq!(row[17].clone().unwrap(), Value::Date(date));
            let json = json!({"key": "value"});
            assert_eq!(row[18].clone().unwrap(), Value::Json(json));
        }

        Ok(())
    }

    async fn test_schema(connection: &mut dyn Connection) -> anyhow::Result<()> {
        let _ = connection
            .execute("CREATE TABLE contacts (id INT PRIMARY KEY, email VARCHAR(20))")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(20))")
            .await?;

        let indexes = connection.indexes(None).await?;
        assert!(indexes.contains(&"PRIMARY".to_string()));

        let indexes = connection.indexes(Some("users")).await?;
        assert!(indexes.contains(&"PRIMARY".to_string()));

        let tables = connection.tables().await?;
        assert!(tables.contains(&"contacts".to_string()));
        assert!(tables.contains(&"users".to_string()));
        Ok(())
    }
}