use crate::metadata;
use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Error::IoError;
use rsql_driver::{Metadata, QueryResult, Result, StatementMetadata, ToSql, UrlExtension, Value};
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, DuckDbDialect};
use std::sync::{Arc, Mutex};
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
        file_type.extensions().contains(&"duckdb")
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

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let values = rsql_driver::to_values(params);
        let duckdb_params = to_duckdb_params(&values);
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(IoError(format!("Error: {error:?}"))),
        };
        let rows = connection
            .execute(sql, duckdb::params_from_iter(duckdb_params.iter()))
            .map_err(|error| IoError(error.to_string()))?;
        Ok(rows as u64)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let values = rsql_driver::to_values(params);
        let duckdb_params = to_duckdb_params(&values);
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(IoError(format!("Error: {error:?}"))),
        };

        let mut statement = connection
            .prepare(sql)
            .map_err(|error| IoError(error.to_string()))?;
        let mut query_rows = statement
            .query(duckdb::params_from_iter(duckdb_params.iter()))
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
                let value = crate::results::convert_to_value(query_row, column_name, index)?;
                row.push(value);
            }
            rows.push(row);
        }

        let query_result = crate::results::DuckDbQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
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

fn to_duckdb_params(values: &[Value]) -> Vec<duckdb::types::Value> {
    values
        .iter()
        .map(|value| match value {
            Value::Null => duckdb::types::Value::Null,
            Value::Bool(v) => duckdb::types::Value::Boolean(*v),
            Value::I8(v) => duckdb::types::Value::TinyInt(*v),
            Value::I16(v) => duckdb::types::Value::SmallInt(*v),
            Value::I32(v) => duckdb::types::Value::Int(*v),
            Value::I64(v) => duckdb::types::Value::BigInt(*v),
            Value::I128(v) => duckdb::types::Value::HugeInt(*v),
            Value::U8(v) => duckdb::types::Value::UTinyInt(*v),
            Value::U16(v) => duckdb::types::Value::USmallInt(*v),
            Value::U32(v) => duckdb::types::Value::UInt(*v),
            Value::U64(v) => duckdb::types::Value::UBigInt(*v),
            Value::F32(v) => duckdb::types::Value::Float(*v),
            Value::F64(v) => duckdb::types::Value::Double(*v),
            Value::String(v) => duckdb::types::Value::Text(v.clone()),
            Value::Bytes(v) => duckdb::types::Value::Blob(v.clone()),
            _ => duckdb::types::Value::Text(value.to_string()),
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use jiff::civil::DateTime;
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
            .query("SELECT id, name FROM users ORDER BY id", &[])
            .await?;

        assert_eq!(query_result.columns(), vec!["id", "name"]);
        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
        );
        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    /// Reference: <https://duckdb.org/docs/sql/data_types/overview.html>
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
        let _ = connection.execute(sql, &[]).await?;

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
        let _ = connection.execute(sql, &[]).await?;

        let sql = indoc! {r"
            SELECT varchar_type, blob_type, bool_type, bit_type,
                   tinyint_type, smallint_type, integer_type, bigint_type,
                   utinyint_type, usmallint_type, uinteger_type, ubigint_type,
                   real_type, double_type, decimal_type,
                   date_type, time_type, timestamp_type
              FROM data_types
        "};
        let mut query_result = connection.query(sql, &[]).await?;

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
            let date = jiff::civil::date(2022, 1, 1);
            assert_eq!(row.get(15).cloned(), Some(Value::Date(date)));
            let time = jiff::civil::time(14, 30, 0, 0);
            assert_eq!(row.get(16).cloned(), Some(Value::Time(time)));
            let date_time = DateTime::from_parts(date, time);
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

    #[tokio::test]
    async fn test_execute_with_params() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let _ = connection
            .execute(
                "CREATE TABLE test_params (id INTEGER, name VARCHAR, score DOUBLE)",
                &[],
            )
            .await?;

        let rows = connection
            .execute(
                "INSERT INTO test_params (id, name, score) VALUES (?, ?, ?)",
                &[&1i32, &"Alice", &95.5f64],
            )
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection
            .query(
                "SELECT id, name, score FROM test_params WHERE id = ?",
                &[&1i32],
            )
            .await?;

        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![
                Value::I32(1),
                Value::String("Alice".to_string()),
                Value::F64(95.5),
            ])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }
}
