use crate::error::{Error, Result};
use crate::metadata::MetadataCache;
use crate::value::Value;
use crate::{sqlite, MemoryQueryResult, Metadata, QueryResult, StatementMetadata};
use anyhow::anyhow;
use async_trait::async_trait;
use rusqlite::types::ValueRef;
use rusqlite::Row;
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, SQLiteDialect};
use std::collections::HashMap;
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
}

#[derive(Debug)]
pub(crate) struct Connection {
    connection: Arc<Mutex<rusqlite::Connection>>,
    metadata_cache: MetadataCache,
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
            rusqlite::Connection::open_in_memory()?
        } else {
            let file = params.get("file").map_or("", |value| value.as_str());
            rusqlite::Connection::open(file)?
        };

        let metadata_cache = MetadataCache::new();

        Ok(Connection {
            connection: Arc::new(Mutex::new(connection)),
            metadata_cache,
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
        let mut statement = connection.prepare(sql)?;
        let rows = statement.execute([])?;
        if let StatementMetadata::DDL = self.parse_sql(sql) {
            self.metadata_cache.invalidate();
        }
        Ok(rows as u64)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        if let Some(metadata) = self.metadata_cache.get() {
            Ok(metadata)
        } else {
            let metadata = sqlite::metadata::get_metadata(self).await?;
            self.metadata_cache.set(metadata.clone());
            Ok(metadata)
        }
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
    use crate::{DriverManager, Value};

    const DATABASE_URL: &str = "rusqlite://?memory=true";

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
            Some(vec![Value::I64(1), Value::String("foo".to_string())])
        );
        assert!(query_result.next().await.is_none());

        let db_metadata = connection.metadata().await?;
        let schema = db_metadata
            .current_schema()
            .expect("expected at least one schema");
        assert!(schema.tables().iter().any(|table| table.name() == "person"));

        connection
            .execute("CREATE TABLE products (id INTEGER, name VARCHAR(20))")
            .await?;
        let db_metadata = connection.metadata().await?;
        let schema = db_metadata
            .current_schema()
            .expect("expected at least one schema");
        assert!(schema
            .tables()
            .iter()
            .any(|table| table.name() == "products"));

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
