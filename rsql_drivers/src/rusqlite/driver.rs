use crate::error::{Error, Result};
use crate::url::UrlExtension;
use crate::value::Value;
use crate::{sqlite, MemoryQueryResult, Metadata, QueryResult, StatementMetadata};
use anyhow::anyhow;
use async_trait::async_trait;
use file_type::FileType;
use rusqlite::types::ValueRef;
use rusqlite::Row;
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, SQLiteDialect};
use std::sync::{Arc, Mutex};
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "rusqlite"
    }

    async fn connect(
        &self,
        url: String,
        _password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let connection = Connection::new(url).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

#[derive(Debug)]
pub(crate) struct Connection {
    url: String,
    connection: Arc<Mutex<rusqlite::Connection>>,
}

impl Connection {
    #[expect(clippy::unused_async)]
    pub(crate) async fn new(url: String) -> Result<Connection> {
        let parsed_url = Url::parse(url.as_str())?;
        let connection = if let Ok(file_name) = parsed_url.to_file() {
            rusqlite::Connection::open(file_name)?
        } else {
            rusqlite::Connection::open_in_memory()?
        };

        Ok(Connection {
            url,
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

#[async_trait]
impl crate::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(Error::IoError(anyhow!("Error: {:?}", error))),
        };
        let mut statement = connection.prepare(sql)?;
        let rows = statement.execute([])?;
        Ok(rows as u64)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        sqlite::metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(Error::IoError(anyhow!("Error: {:?}", error))),
        };

        let mut statement = connection.prepare(sql)?;
        let columns: Vec<String> = statement
            .columns()
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        let mut query_rows = statement.query([])?;
        let mut rows = Vec::new();
        while let Some(query_row) = query_rows.next()? {
            let mut row = Vec::new();
            for (index, _column_name) in columns.iter().enumerate() {
                let value = Self::convert_to_value(query_row, index)?;
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

impl Connection {
    fn convert_to_value(row: &Row, column_index: usize) -> Result<Value> {
        let value = match row.get_ref(column_index)? {
            ValueRef::Null => Value::Null,
            ValueRef::Integer(value) => Value::I64(value),
            ValueRef::Real(value) => Value::F64(value),
            ValueRef::Text(value) => {
                let value = String::from_utf8(value.to_vec())?;
                Value::String(value)
            }
            ValueRef::Blob(value) => Value::Bytes(value.to_vec()),
        };

        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use crate::test::dataset_url;
    use crate::{DriverManager, Value};

    const DATABASE_URL: &str = "rusqlite://";

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        assert_eq!(DATABASE_URL, connection.url());

        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> anyhow::Result<()> {
        let database_url = dataset_url("rusqlite", "users.sqlite3");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&database_url).await?;

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

    /// Ref: https://www.sqlite.org/datatype3.html
    #[tokio::test]
    async fn test_table_data_types() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE t1(t TEXT, nu NUMERIC, i INTEGER, r REAL, no BLOB)")
            .await?;

        let rows = connection
            .execute("INSERT INTO t1 (t, nu, i, r, no) VALUES ('foo', 123, 456, 789.123, x'2a')")
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection.query("SELECT t, nu, i, r, no FROM t1").await?;
        assert_eq!(
            query_result.columns().await,
            vec!["t", "nu", "i", "r", "no"]
        );
        assert_eq!(
            query_result.next().await,
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

    async fn test_data_type(sql: &str) -> anyhow::Result<Option<Value>> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        let mut query_result = connection.query(sql).await?;
        let mut value: Option<Value> = None;

        assert_eq!(query_result.columns().await.len(), 1);

        if let Some(row) = query_result.next().await {
            assert_eq!(row.len(), 1);

            value = row.first().cloned();
        }
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(value)
    }

    #[tokio::test]
    async fn test_data_type_bytes() -> anyhow::Result<()> {
        assert_eq!(
            test_data_type("SELECT x'2a'").await?,
            Some(Value::Bytes(vec![42]))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i64() -> anyhow::Result<()> {
        assert_eq!(
            test_data_type("SELECT 2147483647").await?,
            Some(Value::I64(2_147_483_647))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f64() -> anyhow::Result<()> {
        assert_eq!(
            test_data_type("SELECT 12345.6789").await?,
            Some(Value::F64(12_345.678_9))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_string() -> anyhow::Result<()> {
        assert_eq!(
            test_data_type("SELECT 'foo'").await?,
            Some(Value::String("foo".to_string()))
        );
        Ok(())
    }
}
