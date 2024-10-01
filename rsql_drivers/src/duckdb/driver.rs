use crate::duckdb::metadata;
use crate::error::{Error, Result};
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, Metadata, QueryResult};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use duckdb::types::{TimeUnit, ValueRef};
use duckdb::Row;
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
    #[allow(clippy::unused_async)]
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

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
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
                let column_name = columns.get(index).expect("no column");
                let value = Self::convert_to_value(query_row, column_name, index)?;
                row.push(value);
            }
            rows.push(crate::Row::new(row));
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn ddl_keywords(&self) -> Vec<&'static str> {
        vec!["CREATE", "ALTER", "DROP", "ANALYZE", "VACUUM", "IMPORT"]
    }
}

impl Connection {
    fn convert_to_value(row: &Row, column_name: &String, column_index: usize) -> Result<Value> {
        let value_ref = row.get_ref(column_index)?;
        let value = match value_ref {
            ValueRef::Null => Value::Null,
            ValueRef::Boolean(value) => Value::Bool(value),
            ValueRef::TinyInt(value) => Value::I8(value),
            ValueRef::SmallInt(value) => Value::I16(value),
            ValueRef::Int(value) => Value::I32(value),
            ValueRef::BigInt(value) => Value::I64(value),
            ValueRef::HugeInt(value) => Value::I128(value),
            ValueRef::UTinyInt(value) => Value::U8(value),
            ValueRef::USmallInt(value) => Value::U16(value),
            ValueRef::UInt(value) => Value::U32(value),
            ValueRef::UBigInt(value) => Value::U64(value),
            ValueRef::Float(value) => Value::F32(value),
            ValueRef::Double(value) => Value::F64(value),
            ValueRef::Decimal(value) => Value::String(value.to_string()),
            ValueRef::Text(value) => {
                let value = String::from_utf8(value.to_vec())?;
                Value::String(value)
            }
            ValueRef::Blob(value) => Value::Bytes(value.to_vec()),
            ValueRef::Date32(value) => {
                let start_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("invalid date");
                let delta = TimeDelta::days(i64::from(value));
                let date = start_date.add(delta);
                Value::Date(date)
            }
            ValueRef::Time64(unit, value) => {
                let start_time = NaiveTime::from_hms_opt(0, 0, 0).expect("invalid time");
                let duration = match unit {
                    TimeUnit::Second => Duration::from_secs(u64::try_from(value)?),
                    TimeUnit::Millisecond => Duration::from_millis(u64::try_from(value)?),
                    TimeUnit::Microsecond => Duration::from_micros(u64::try_from(value)?),
                    TimeUnit::Nanosecond => Duration::from_nanos(u64::try_from(value)?),
                };
                let time = start_time.add(duration);
                Value::Time(time)
            }
            ValueRef::Timestamp(unit, value) => {
                let start_date = NaiveDate::from_ymd_opt(1970, 1, 1).expect("invalid date");
                let start_time = NaiveTime::from_hms_opt(0, 0, 0).expect("invalid time");
                let start_date_time = NaiveDateTime::new(start_date, start_time);
                let duration = match unit {
                    TimeUnit::Second => Duration::from_secs(u64::try_from(value)?),
                    TimeUnit::Millisecond => Duration::from_millis(u64::try_from(value)?),
                    TimeUnit::Microsecond => Duration::from_micros(u64::try_from(value)?),
                    TimeUnit::Nanosecond => Duration::from_nanos(u64::try_from(value)?),
                };
                let date_time = start_date_time.add(duration);
                Value::DateTime(date_time)
            }
            _ => {
                let data_type = value_ref.data_type();
                return Err(UnsupportedColumnType {
                    column_name: column_name.to_string(),
                    column_type: data_type.to_string(),
                });
            }
        };

        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use crate::{DriverManager, Row, Value};
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
        assert_eq!(
            query_result.next().await,
            Some(Row::new(vec![
                Value::I32(1),
                Value::String("foo".to_string())
            ]))
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    /// Ref: https://duckdb.org/docs/sql/data_types/overview.html
    #[tokio::test]
    async fn test_table_data_types() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        let sql = indoc! {r"
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
        "};
        let _ = connection.execute(sql).await?;

        let sql = indoc! {r"
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
        "};
        let _ = connection.execute(sql).await?;

        let sql = indoc! {r"
            SELECT varchar_type, blob_type, bool_type, bit_type,
                   tinyint_type, smallint_type, integer_type, bigint_type,
                   utinyint_type, usmallint_type, uinteger_type, ubigint_type,
                   real_type, double_type, decimal_type,
                   date_type, time_type, timestamp_type
              FROM data_types
        "};
        let mut query_result = connection.query(sql).await?;

        if let Some(row) = query_result.next().await {
            assert_eq!(
                row.first().cloned().expect("no value"),
                Value::String("a".to_string())
            );
            assert_eq!(
                row.get(1).cloned().expect("no value"),
                Value::Bytes("foo".as_bytes().to_vec())
            );
            assert_eq!(row.get(2).cloned(), Some(Value::Bool(true)));
            assert_eq!(row.get(3).cloned(), Some(Value::Bytes(vec![2, 234])));
            assert_eq!(row.get(4).cloned(), Some(Value::I8(127)));
            assert_eq!(row.get(5).cloned(), Some(Value::I16(32_767)));
            assert_eq!(row.get(6).cloned(), Some(Value::I32(2_147_483_647)));
            assert_eq!(
                row.get(7).cloned(),
                Some(Value::I64(9_223_372_036_854_775_807))
            );
            assert_eq!(row.get(8).cloned(), Some(Value::U8(255)));
            assert_eq!(row.get(9).cloned(), Some(Value::U16(65_535)));
            assert_eq!(row.get(10).cloned(), Some(Value::U32(4_294_967_295)));
            assert_eq!(
                row.get(11).cloned(),
                Some(Value::U64(18_446_744_073_709_551_615))
            );
            assert_eq!(row.get(12).cloned(), Some(Value::F32(123.45)));
            assert_eq!(row.get(13).cloned(), Some(Value::F64(123.0)));
            assert_eq!(
                row.get(14).cloned(),
                Some(Value::String("123.00".to_string()))
            );
            let date = NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid date");
            assert_eq!(row.get(15).cloned(), Some(Value::Date(date)));
            let time = NaiveTime::from_hms_opt(14, 30, 00).expect("invalid time");
            assert_eq!(row.get(16).cloned(), Some(Value::Time(time)));
            let date_time =
                NaiveDateTime::parse_from_str("2022-01-01 14:30:00", "%Y-%m-%d %H:%M:%S")?;
            assert_eq!(row.get(17).cloned(), Some(Value::DateTime(date_time)));
        }

        Ok(())
    }
}
