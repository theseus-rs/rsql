use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Error::IoError;
use rsql_driver::{Metadata, QueryResult, Result, StatementMetadata, ToSql, UrlExtension, Value};
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, SQLiteDialect};
use std::sync::{Arc, Mutex};
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "rusqlite"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let connection = Connection::new(url).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Connection {
    url: String,
    connection: Arc<Mutex<rusqlite::Connection>>,
}

impl Connection {
    #[expect(clippy::unused_async)]
    pub(crate) async fn new(url: &str) -> Result<Connection> {
        let parsed_url = Url::parse(url)?;
        let connection = if let Ok(file_name) = parsed_url.to_file() {
            rusqlite::Connection::open(file_name).map_err(|error| IoError(error.to_string()))?
        } else {
            rusqlite::Connection::open_in_memory().map_err(|error| IoError(error.to_string()))?
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
        let rusqlite_params = to_rusqlite_params(&values);
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(IoError(error.to_string())),
        };
        let mut statement = connection
            .prepare(sql)
            .map_err(|error| IoError(error.to_string()))?;
        let rows = statement
            .execute(rusqlite::params_from_iter(rusqlite_params.iter()))
            .map_err(|error| IoError(error.to_string()))?;
        Ok(rows as u64)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let values = rsql_driver::to_values(params);
        let rusqlite_params = to_rusqlite_params(&values);
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(IoError(error.to_string())),
        };

        let mut statement = connection
            .prepare(sql)
            .map_err(|error| IoError(error.to_string()))?;
        let columns: Vec<String> = statement
            .columns()
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        let mut query_rows = statement
            .query(rusqlite::params_from_iter(rusqlite_params.iter()))
            .map_err(|error| IoError(error.to_string()))?;
        let mut rows = Vec::new();
        while let Some(query_row) = query_rows
            .next()
            .map_err(|error| IoError(error.to_string()))?
        {
            let mut row = Vec::new();
            for (index, _column_name) in columns.iter().enumerate() {
                let value = crate::results::convert_to_value(query_row, index)?;
                row.push(value);
            }
            rows.push(row);
        }

        let query_result = crate::results::RusqliteQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        rsql_driver_sqlite::get_metadata(self).await
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(SQLiteDialect {})
    }

    fn match_statement(&self, statement: &Statement) -> StatementMetadata {
        let default = self.default_match_statement(statement);
        match default {
            StatementMetadata::Unknown => match statement {
                // missing: DETACH DATABASE
                Statement::CreateVirtualTable { .. } | Statement::AttachDatabase { .. } => {
                    StatementMetadata::DDL
                }
                _ => StatementMetadata::Unknown,
            },
            other => other,
        }
    }
}

fn to_rusqlite_params(values: &[Value]) -> Vec<rusqlite::types::Value> {
    values
        .iter()
        .map(|value| match value {
            Value::Null => rusqlite::types::Value::Null,
            Value::Bool(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::I8(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::I16(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::I32(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::I64(v) => rusqlite::types::Value::Integer(*v),
            Value::U8(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::U16(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::U32(v) => rusqlite::types::Value::Integer(i64::from(*v)),
            Value::U64(v) => rusqlite::types::Value::Integer(*v as i64),
            Value::F32(v) => rusqlite::types::Value::Real(f64::from(*v)),
            Value::F64(v) => rusqlite::types::Value::Real(*v),
            Value::String(v) => rusqlite::types::Value::Text(v.clone()),
            Value::Bytes(v) => rusqlite::types::Value::Blob(v.clone()),
            _ => rusqlite::types::Value::Text(value.to_string()),
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::Driver;
    use rsql_driver_test_utils::dataset_url;

    const DATABASE_URL: &str = "rusqlite://";

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
        let database_url = dataset_url("rusqlite", "users.sqlite3");
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

    /// Reference: <https://www.sqlite.org/datatype3.html>
    #[tokio::test]
    async fn test_table_data_types() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let _ = connection
            .execute(
                "CREATE TABLE t1(t TEXT, nu NUMERIC, i INTEGER, r REAL, no BLOB)",
                &[],
            )
            .await?;

        let rows = connection
            .execute(
                "INSERT INTO t1 (t, nu, i, r, no) VALUES ('foo', 123, 456, 789.123, x'2a')",
                &[],
            )
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection
            .query("SELECT t, nu, i, r, no FROM t1", &[])
            .await?;
        assert_eq!(query_result.columns(), vec!["t", "nu", "i", "r", "no"]);
        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![
                Value::String("foo".to_string()),
                Value::I64(123),
                Value::I64(456),
                Value::F64(789.123),
                Value::Bytes(vec![42])
            ])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> Result<Option<Value>> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;
        let mut query_result = connection.query(sql, &[]).await?;
        let mut value: Option<Value> = None;

        assert_eq!(query_result.columns().len(), 1);

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
        assert_eq!(
            test_data_type("SELECT x'2a'").await?,
            Some(Value::Bytes(vec![42]))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i64() -> Result<()> {
        assert_eq!(
            test_data_type("SELECT 2147483647").await?,
            Some(Value::I64(2_147_483_647))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f64() -> Result<()> {
        assert_eq!(
            test_data_type("SELECT 12345.6789").await?,
            Some(Value::F64(12_345.678_9))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_string() -> Result<()> {
        assert_eq!(
            test_data_type("SELECT 'foo'").await?,
            Some(Value::String("foo".to_string()))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_params() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let _ = connection
            .execute(
                "CREATE TABLE test_params (id INTEGER, name TEXT, score REAL)",
                &[],
            )
            .await?;

        let rows = connection
            .execute(
                "INSERT INTO test_params (id, name, score) VALUES (?, ?, ?)",
                &[&1i64, &"Alice", &95.5f64],
            )
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection
            .query(
                "SELECT id, name, score FROM test_params WHERE id = ?",
                &[&1i64],
            )
            .await?;

        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![
                Value::I64(1),
                Value::String("Alice".to_string()),
                Value::F64(95.5),
            ])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }
}
