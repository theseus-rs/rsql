use crate::metadata;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeDelta};
use duckdb::Row;
use duckdb::types::{TimeUnit, ValueRef};
use file_type::FileType;
use rsql_driver::Error::{IoError, UnsupportedColumnType};
use rsql_driver::{
    MemoryQueryResult, Metadata, QueryResult, Result, StatementMetadata, UrlExtension, Value,
};
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, DuckDbDialect};
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "duckdb"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let connection = Connection::new(url).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type
            .media_types()
            .contains(&"application/vnd.duckdb.file")
    }
}

#[derive(Debug)]
pub struct Connection {
    url: String,
    connection: Arc<Mutex<duckdb::Connection>>,
}

impl Connection {
    #[expect(clippy::unused_async)]
    pub(crate) async fn new(url: &str) -> Result<Connection> {
        let parsed_url = Url::parse(url)?;
        let connection = if let Ok(file_name) = parsed_url.to_file() {
            duckdb::Connection::open(file_name).map_err(|error| IoError(error.to_string()))?
        } else {
            duckdb::Connection::open_in_memory().map_err(|error| IoError(error.to_string()))?
        };

        Ok(Connection {
            url: url.to_string(),
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(IoError(format!("Error: {error:?}"))),
        };
        let rows = connection
            .execute(sql, [])
            .map_err(|error| IoError(error.to_string()))?;
        Ok(rows as u64)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(IoError(format!("Error: {error:?}"))),
        };

        let mut statement = connection
            .prepare(sql)
            .map_err(|error| IoError(error.to_string()))?;
        let mut query_rows = statement
            .query([])
            .map_err(|error| IoError(error.to_string()))?;
        let columns = query_rows.as_ref().expect("no rows").column_names();
        let mut rows = Vec::new();
        while let Some(query_row) = query_rows
            .next()
            .map_err(|error| IoError(error.to_string()))?
        {
            let mut row = Vec::new();
            for (index, _column_name) in columns.iter().enumerate() {
                let column_name = columns.get(index).expect("no column");
                let value = Self::convert_to_value(query_row, column_name, index)?;
                row.push(value);
            }
            rows.push(row);
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(DuckDbDialect {})
    }

    fn match_statement(&self, statement: &Statement) -> StatementMetadata {
        let default = self.default_match_statement(statement);
        match default {
            StatementMetadata::Unknown => match statement {
                Statement::Install { .. }
                | Statement::AttachDuckDBDatabase { .. }
                | Statement::CreateMacro { .. }
                | Statement::CreateSecret { .. }
                | Statement::DetachDuckDBDatabase { .. }
                | Statement::Load { .. } => StatementMetadata::DDL,
                _ => StatementMetadata::Unknown,
            },
            other => other,
        }
    }
}

impl Connection {
    fn convert_to_value(row: &Row, column_name: &String, column_index: usize) -> Result<Value> {
        let value_ref = row
            .get_ref(column_index)
            .map_err(|error| IoError(error.to_string()))?;
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
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use indoc::indoc;
    use rsql_driver::Driver;
    use rsql_driver_test_utils::dataset_url;

    const DATABASE_URL: &str = "duckdb://";

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
        let database_url = dataset_url("duckdb", "users.duckdb");
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url).await?;

        let mut query_result = connection
            .query("SELECT id, name FROM users ORDER BY id")
            .await?;

        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
        );
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    /// Ref: https://duckdb.org/docs/sql/data_types/overview.html
    #[tokio::test]
    async fn test_table_data_types() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;
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
                NaiveDateTime::parse_from_str("2022-01-01 14:30:00", "%Y-%m-%d %H:%M:%S")
                    .map_err(|error| IoError(error.to_string()))?;
            assert_eq!(row.get(17).cloned(), Some(Value::DateTime(date_time)));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_dialect() -> Result<()> {
        let driver = crate::Driver;
        let connection = driver.connect(DATABASE_URL).await?;
        let dialect = connection.dialect();

        assert!(dialect.is_delimited_identifier_start('"'));
        assert!(!dialect.is_delimited_identifier_start('\''));
        assert!(dialect.is_identifier_start('a'));
        assert!(dialect.is_identifier_part('_'));

        Ok(())
    }
    #[tokio::test]
    async fn test_parse_sql() -> Result<()> {
        let driver = crate::Driver;
        let connection = driver.connect(DATABASE_URL).await?;

        let ddl_sql_statements = [
            "INSTALL extension_name",
            "ATTACH 'file.db'",
            "CREATE MACRO add(a, b) AS a + b",
            "CREATE SECRET secret_name ( TYPE secret_type )",
            "DETACH file",
            "LOAD extension_name",
        ];

        for sql in ddl_sql_statements {
            let statement_meta = connection.parse_sql(sql);
            assert!(matches!(statement_meta, StatementMetadata::DDL));
        }
        Ok(())
    }
}
