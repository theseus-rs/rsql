use crate::configuration::Configuration;
use crate::drivers::error::{Error, Result};
use crate::drivers::value::Value;
use crate::drivers::{MemoryQueryResult, Results};
use anyhow::anyhow;
use async_trait::async_trait;
use indoc::indoc;
use rusqlite::types::ValueRef;
use rusqlite::Row;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::drivers::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "rusqlite"
    }

    async fn connect(
        &self,
        configuration: &Configuration,
        url: String,
        _password: Option<String>,
    ) -> Result<Box<dyn crate::drivers::Connection>> {
        let connection = Connection::new(configuration, url).await?;
        Ok(Box::new(connection))
    }
}

#[derive(Debug)]
pub(crate) struct Connection {
    connection: Arc<Mutex<rusqlite::Connection>>,
}

impl Connection {
    pub(crate) async fn new(_configuration: &Configuration, url: String) -> Result<Connection> {
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

        Ok(Connection {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

#[async_trait]
impl crate::drivers::Connection for Connection {
    async fn execute(&self, sql: &str) -> Result<Results> {
        let connection = match self.connection.lock() {
            Ok(connection) => connection,
            Err(error) => return Err(Error::IoError(anyhow!("Error: {:?}", error))),
        };
        let mut statement = connection.prepare(sql)?;
        let rows = statement.execute([])?;
        Ok(Results::Execute(rows as u64))
    }

    async fn indexes<'table>(&mut self, table: Option<&'table str>) -> Result<Vec<String>> {
        let mut sql = indoc! {r#"
            SELECT name
              FROM sqlite_master
             WHERE type = 'index'
        "#}
        .to_string();
        if table.is_some() {
            sql = format!("{sql} AND tbl_name = ?1");
        }
        sql = format!("{sql} ORDER BY name");

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

    async fn query(&self, sql: &str, limit: u64) -> Result<Results> {
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
                let value = self.convert_to_value(query_row, index)?;
                row.push(value);
            }
            rows.push(row);

            if limit > 0 && rows.len() >= limit as usize {
                break;
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Results::Query(Box::new(query_result)))
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
        let results = self.query(sql, 0).await?;
        let mut tables = Vec::new();

        if let Results::Query(query_results) = results {
            for row in query_results.rows().await {
                if let Some(data) = &row[0] {
                    tables.push(data.to_string());
                }
            }
        }

        Ok(tables)
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Connection {
    fn convert_to_value(&self, row: &Row, column_index: usize) -> Result<Option<Value>> {
        let value = match row.get_ref(column_index)? {
            ValueRef::Null => None,
            ValueRef::Integer(value) => Some(Value::I64(value)),
            ValueRef::Real(value) => Some(Value::F64(value)),
            ValueRef::Text(value) => {
                let value = match String::from_utf8(value.to_vec()) {
                    Ok(value) => value,
                    Err(error) => return Err(Error::IoError(anyhow!("Error: {:?}", error))),
                };
                Some(Value::String(value))
            }
            ValueRef::Blob(value) => Some(Value::Bytes(value.to_vec())),
        };

        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use crate::configuration::Configuration;
    use crate::drivers::{DriverManager, Results, Value};

    const DATABASE_URL: &str = "rusqlite://?memory=true";

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;
        connection.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_limit_rows() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let connection = driver_manager.connect(&configuration, DATABASE_URL).await?;
        let results = connection.query("SELECT 1 UNION ALL SELECT 2", 1).await?;
        assert!(results.is_query());
        if let Results::Query(query_result) = results {
            assert_eq!(query_result.rows().await.len(), 1);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> anyhow::Result<()> {
        let configuration = &Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE person (id INTEGER, name TEXT)")
            .await?;

        let execute_results = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        if let Results::Execute(rows) = execute_results {
            assert_eq!(rows, 1);
        }

        let results = connection.query("SELECT id, name FROM person", 0).await?;
        if let Results::Query(query_result) = results {
            assert_eq!(query_result.columns().await, vec!["id", "name"]);
            assert_eq!(query_result.rows().await.len(), 1);
            match query_result.rows().await.get(0) {
                Some(row) => {
                    assert_eq!(row.len(), 2);

                    if let Some(Value::I64(id)) = &row[0] {
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

        connection.stop().await?;
        Ok(())
    }

    /// Ref: https://www.sqlite.org/datatype3.html
    #[tokio::test]
    async fn test_table_data_types() -> anyhow::Result<()> {
        let configuration = &Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE t1(t TEXT, nu NUMERIC, i INTEGER, r REAL, no BLOB)")
            .await?;

        let execute_results = connection
            .execute("INSERT INTO t1 (t, nu, i, r, no) VALUES ('foo', 123, 456, 789.123, x'2a')")
            .await?;
        if let Results::Execute(rows) = execute_results {
            assert_eq!(rows, 1);
        }

        let results = connection
            .query("SELECT t, nu, i, r, no FROM t1", 0)
            .await?;
        if let Results::Query(query_result) = results {
            assert_eq!(
                query_result.columns().await,
                vec!["t", "nu", "i", "r", "no"]
            );
            assert_eq!(query_result.rows().await.len(), 1);
            match query_result.rows().await.get(0) {
                Some(row) => {
                    assert_eq!(row.len(), 5);

                    if let Some(Value::String(value)) = &row[0] {
                        assert_eq!(value, "foo");
                    } else {
                        assert!(false);
                    }

                    if let Some(Value::I64(value)) = &row[1] {
                        assert_eq!(*value, 123);
                    } else {
                        assert!(false);
                    }

                    if let Some(Value::I64(value)) = &row[2] {
                        assert_eq!(*value, 456);
                    } else {
                        assert!(false);
                    }

                    if let Some(Value::F64(value)) = &row[3] {
                        assert_eq!(*value, 789.123);
                    } else {
                        assert!(false);
                    }

                    if let Some(Value::Bytes(value)) = &row[4] {
                        assert_eq!(*value, vec![42]);
                    } else {
                        assert!(false);
                    }
                }
                None => assert!(false),
            }
        }

        connection.stop().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> anyhow::Result<Option<Value>> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;
        let results = connection.query(sql, 0).await?;
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
        match test_data_type("SELECT x'2a'").await? {
            Some(value) => assert_eq!(value, Value::Bytes(vec![42])),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i64() -> anyhow::Result<()> {
        match test_data_type("SELECT 2147483647").await? {
            Some(value) => assert_eq!(value, Value::I64(2_147_483_647)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f64() -> anyhow::Result<()> {
        match test_data_type("SELECT 12345.6789").await? {
            Some(value) => assert_eq!(value, Value::F64(12_345.6789)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_string() -> anyhow::Result<()> {
        match test_data_type("SELECT 'foo'").await? {
            Some(value) => assert_eq!(value, Value::String("foo".to_string())),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_schema() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;

        let indexes = connection.indexes(None).await?;
        assert_eq!(
            indexes,
            vec!["sqlite_autoindex_contacts_1", "sqlite_autoindex_users_1"]
        );

        let indexes = connection.indexes(Some("users")).await?;
        assert_eq!(indexes, vec!["sqlite_autoindex_users_1"]);

        let tables = connection.tables().await?;
        assert_eq!(tables, vec!["contacts", "users"]);

        connection.stop().await?;
        Ok(())
    }
}
