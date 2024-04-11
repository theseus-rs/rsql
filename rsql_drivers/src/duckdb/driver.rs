use crate::error::{Error, Result};
use crate::value::Value;
use crate::{MemoryQueryResult, QueryResult};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use duckdb::types::{TimeUnit, ValueRef};
use duckdb::Row;
use indoc::indoc;
use std::collections::HashMap;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "duckdb"
    }

    async fn connect(
        &self,
        url: String,
        _password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let connection = Connection::new(url).await?;
        Ok(Box::new(connection))
    }
}

#[derive(Debug)]
pub(crate) struct Connection {
    connection: Arc<Mutex<duckdb::Connection>>,
}

impl Connection {
    pub(crate) async fn new(url: String) -> Result<Connection> {
        let parsed_url = Url::parse(url.as_str())?;
        let mut params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let memory = params
            .remove("memory")
            .map_or(false, |value| value == "true");

        let connection = if memory {
            duckdb::Connection::open_in_memory()?
        } else {
            let file = params.get("file").map_or("", |value| value.as_str());
            duckdb::Connection::open(file)?
        };

        Ok(Connection {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

#[async_trait]
impl crate::Connection for Connection {
    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(Error::IoError(anyhow!("Error: {:?}", error))),
        };
        let rows = connection.execute(sql, [])?;
        Ok(rows as u64)
    }

    async fn indexes<'table>(&mut self, table: Option<&'table str>) -> Result<Vec<String>> {
        let mut sql = indoc! {r#"
            SELECT index_name
              FROM duckdb_indexes()
        "#}
        .to_string();
        if table.is_some() {
            sql = format!("{sql} WHERE table_name = ?1");
        }
        sql = format!("{sql} ORDER BY index_name");

        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(Error::IoError(anyhow!("Error: {:?}", error))),
        };
        let mut statement = connection.prepare(sql.as_str())?;

        let mut query_rows = match table {
            Some(table) => statement.query([table])?,
            None => statement.query([])?,
        };

        let mut indexes = Vec::new();
        while let Some(query_row) = query_rows.next()? {
            if let Some(value) = self.convert_to_value(query_row, 0)? {
                indexes.push(value.to_string());
            }
        }

        Ok(indexes)
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(Error::IoError(anyhow!("Error: {:?}", error))),
        };

        let mut statement = connection.prepare(sql)?;
        let mut query_rows = statement.query([])?;
        let columns = query_rows.as_ref().expect("no rows").column_names();
        let mut rows = Vec::new();
        while let Some(query_row) = query_rows.next()? {
            let mut row = Vec::new();
            for (index, _column_name) in columns.iter().enumerate() {
                let value = self.convert_to_value(query_row, index)?;
                row.push(value);
            }
            rows.push(crate::Row::new(row));
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = indoc! { r#"
            SELECT table_name
              FROM information_schema.tables
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
        Ok(())
    }
}

impl Connection {
    fn convert_to_value(&self, row: &Row, column_index: usize) -> Result<Option<Value>> {
        let value = match row.get_ref(column_index)? {
            ValueRef::Null => None,
            ValueRef::Boolean(value) => Some(Value::Bool(value)),
            ValueRef::TinyInt(value) => Some(Value::I8(value)),
            ValueRef::SmallInt(value) => Some(Value::I16(value)),
            ValueRef::Int(value) => Some(Value::I32(value)),
            ValueRef::BigInt(value) => Some(Value::I64(value)),
            ValueRef::HugeInt(value) => Some(Value::I128(value)),
            ValueRef::UTinyInt(value) => Some(Value::U8(value)),
            ValueRef::USmallInt(value) => Some(Value::U16(value)),
            ValueRef::UInt(value) => Some(Value::U32(value)),
            ValueRef::UBigInt(value) => Some(Value::U64(value)),
            ValueRef::Float(value) => Some(Value::F32(value)),
            ValueRef::Double(value) => Some(Value::F64(value)),
            ValueRef::Decimal(value) => Some(Value::String(value.to_string())),
            ValueRef::Text(value) => {
                let value = match String::from_utf8(value.to_vec()) {
                    Ok(value) => value,
                    Err(error) => return Err(Error::IoError(anyhow!("Error: {:?}", error))),
                };
                Some(Value::String(value))
            }
            ValueRef::Blob(value) => Some(Value::Bytes(value.to_vec())),
            ValueRef::Date32(value) => {
                let start_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("invalid date");
                let delta = TimeDelta::days(value as i64);
                let date = start_date.add(delta);
                Some(Value::Date(date))
            }
            ValueRef::Time64(unit, value) => {
                let start_time = NaiveTime::from_hms_opt(0, 0, 0).expect("invalid time");
                let duration = match unit {
                    TimeUnit::Second => Duration::from_secs(value as u64),
                    TimeUnit::Millisecond => Duration::from_millis(value as u64),
                    TimeUnit::Microsecond => Duration::from_micros(value as u64),
                    TimeUnit::Nanosecond => Duration::from_nanos(value as u64),
                };
                let time = start_time.add(duration);
                Some(Value::Time(time))
            }
            ValueRef::Timestamp(unit, value) => {
                let start_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("invalid date");
                let start_time = NaiveTime::from_hms_opt(0, 0, 0).expect("invalid time");
                let start_date_time = NaiveDateTime::new(start_date, start_time);
                let duration = match unit {
                    TimeUnit::Second => Duration::from_secs(value as u64),
                    TimeUnit::Millisecond => Duration::from_millis(value as u64),
                    TimeUnit::Microsecond => Duration::from_micros(value as u64),
                    TimeUnit::Nanosecond => Duration::from_nanos(value as u64),
                };
                let date_time = start_date_time.add(duration);
                Some(Value::DateTime(date_time))
            }
        };

        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use crate::{DriverManager, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use indoc::indoc;

    const DATABASE_URL: &str = "duckdb://?memory=true";

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
            .execute("CREATE TABLE person (id INTEGER, name TEXT)")
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

    /// Ref: https://duckdb.org/docs/sql/data_types/overview.html
    #[tokio::test]
    async fn test_table_data_types() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        let sql = indoc! {r#"
            CREATE TABLE data_types (
                varchar_type VARCHAR,
                blob_type BLOB,
                bool_type BOOL,
                bit_type BIT,
                tinyint_type TINYINT,
                smallint_type SMALLINT,
                integer_type INTEGER,
                bigint_type BIGINT,
                utinyint_type UTINYINT,
                usmallint_type USMALLINT,
                uinteger_type UINTEGER,
                ubigint_type UBIGINT,
                real_type REAL,
                double_type DOUBLE,
                decimal_type DECIMAL(5,2),
                date_type DATE,
                time_type TIME,
                timestamp_type TIMESTAMP
            )
        "#};
        let _ = connection.execute(sql).await?;

        let sql = indoc! {r#"
            INSERT INTO data_types (
                varchar_type, blob_type, bool_type, bit_type,
                tinyint_type, smallint_type, integer_type, bigint_type,
                utinyint_type, usmallint_type, uinteger_type, ubigint_type,
                real_type, double_type, decimal_type,
                date_type, time_type, timestamp_type
            ) VALUES (
                 'a', 'foo', true, '101010',
                 127, 32767, 2147483647, 9223372036854775807,
                 255, 65535, 4294967295, 18446744073709551615,
                 123.45, 123.0, 123.0,
                 '2022-01-01', '14:30:00', '2022-01-01 14:30:00'
            )
        "#};
        let _ = connection.execute(sql).await?;

        let sql = indoc! {r#"
            SELECT varchar_type, blob_type, bool_type, bit_type,
                   tinyint_type, smallint_type, integer_type, bigint_type,
                   utinyint_type, usmallint_type, uinteger_type, ubigint_type,
                   real_type, double_type, decimal_type,
                   date_type, time_type, timestamp_type
              FROM data_types
        "#};
        let mut query_result = connection.query(sql).await?;

        if let Some(row) = query_result.next().await {
            assert_eq!(row.get(0).cloned().unwrap(), Value::String("a".to_string()));
            assert_eq!(
                row.get(1).cloned().unwrap(),
                Value::Bytes("foo".as_bytes().to_vec())
            );
            assert_eq!(row.get(2).cloned().unwrap(), Value::Bool(true));
            assert_eq!(row.get(3).cloned().unwrap(), Value::Bytes(vec![2, 234]));
            assert_eq!(row.get(4).cloned().unwrap(), Value::I8(127));
            assert_eq!(row.get(5).cloned().unwrap(), Value::I16(32_767));
            assert_eq!(row.get(6).cloned().unwrap(), Value::I32(2_147_483_647));
            assert_eq!(
                row.get(7).cloned().unwrap(),
                Value::I64(9_223_372_036_854_775_807)
            );
            assert_eq!(row.get(8).cloned().unwrap(), Value::U8(255));
            assert_eq!(row.get(9).cloned().unwrap(), Value::U16(65_535));
            assert_eq!(row.get(10).cloned().unwrap(), Value::U32(4_294_967_295));
            assert_eq!(
                row.get(11).cloned().unwrap(),
                Value::U64(18_446_744_073_709_551_615)
            );
            assert_eq!(row.get(12).cloned().unwrap(), Value::F32(123.45));
            assert_eq!(row.get(13).cloned().unwrap(), Value::F64(123.0));
            assert_eq!(
                row.get(14).cloned().unwrap(),
                Value::String("123.00".to_string())
            );
            let date = NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid date");
            assert_eq!(row.get(15).cloned().unwrap(), Value::Date(date));
            let time = NaiveTime::from_hms_opt(14, 30, 00).expect("invalid time");
            assert_eq!(row.get(16).cloned().unwrap(), Value::Time(time));
            let date_time =
                NaiveDateTime::parse_from_str("2022-01-01 14:30:00", "%Y-%m-%d %H:%M:%S")?;
            assert_eq!(row.get(17).cloned().unwrap(), Value::DateTime(date_time));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_schema() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;
        let _ = connection
            .execute("CREATE INDEX contacts_email_idx ON contacts (email)")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;
        let _ = connection
            .execute("CREATE INDEX users_email_idx ON users (email)")
            .await?;

        let tables = connection.tables().await?;
        assert_eq!(tables, vec!["contacts", "users"]);

        let indexes = connection.indexes(None).await?;
        assert_eq!(indexes, vec!["contacts_email_idx", "users_email_idx"]);

        let indexes = connection.indexes(Some("users")).await?;
        assert_eq!(indexes, vec!["users_email_idx"]);

        connection.close().await?;
        Ok(())
    }
}
