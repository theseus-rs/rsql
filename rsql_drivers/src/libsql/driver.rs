use crate::error::Result;
use crate::value::Value;
use crate::{MemoryQueryResult, QueryResult, Row};
use async_trait::async_trait;
use indoc::indoc;
use libsql::replication::Frames;
use libsql::Builder;
use std::collections::HashMap;
use std::fmt::Debug;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "libsql"
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

pub(crate) struct Connection {
    connection: libsql::Connection,
    url: String,
}

impl Connection {
    pub(crate) async fn new(url: String) -> Result<Connection> {
        let parsed_url = Url::parse(url.as_str())?;
        let mut params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let memory = params
            .remove("memory")
            .map_or(false, |value| value == "true");
        let file = params.get("file").map_or("", |value| value.as_str());

        let database = if memory {
            Builder::new_local(":memory:").build().await?
        } else if !file.is_empty() {
            let db = Builder::new_local_replica(file).build().await?;
            let frames = Frames::Vec(vec![]);
            db.sync_frames(frames).await?;
            db
        } else {
            let host = parsed_url.host().expect("Host is required").to_string();
            let auth_token = params.get("auth_token").map_or("", |value| value.as_str());
            let database_url = format!("libsql://{}", host);
            Builder::new_remote(database_url, auth_token.to_string())
                .build()
                .await?
        };

        let connection = database.connect()?;

        Ok(Connection { connection, url })
    }
}

#[async_trait]
impl crate::Connection for Connection {
    async fn execute(&self, sql: &str) -> Result<u64> {
        let rows = self.connection.execute(sql, ()).await?;
        Ok(rows)
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

        let mut statement = self.connection.prepare(sql.as_str()).await?;

        let mut query_rows = match table {
            Some(table) => statement.query([table]).await?,
            None => statement.query(()).await?,
        };

        let mut indexes = Vec::new();
        while let Some(query_row) = query_rows.next().await? {
            if let Some(value) = self.convert_to_value(&query_row, 0)? {
                indexes.push(value.to_string());
            }
        }

        Ok(indexes)
    }

    async fn query(&self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let mut statement = self.connection.prepare(sql).await?;
        let columns: Vec<String> = statement
            .columns()
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        let mut query_rows = statement.query(()).await?;
        let mut rows = Vec::new();
        while let Some(query_row) = query_rows.next().await? {
            let mut row = Vec::new();
            for (index, _column_name) in columns.iter().enumerate() {
                let value = self.convert_to_value(&query_row, index as i32)?;
                row.push(value);
            }
            rows.push(Row::new(row));
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
        let mut query_results = self.query(sql).await?;
        let mut tables = Vec::new();

        while let Some(row) = query_results.next().await {
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
    fn convert_to_value(&self, row: &libsql::Row, column_index: i32) -> Result<Option<Value>> {
        let value = match row.get_value(column_index)? {
            libsql::Value::Null => None,
            libsql::Value::Integer(value) => Some(Value::I64(value)),
            libsql::Value::Real(value) => Some(Value::F64(value)),
            libsql::Value::Text(value) => Some(Value::String(value)),
            libsql::Value::Blob(value) => Some(Value::Bytes(value.to_vec())),
        };

        Ok(value)
    }
}

impl Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("url", &self.url)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use crate::{DriverManager, Value};

    const DATABASE_URL: &str = "libsql://?memory=true";

    #[tokio::test]
    async fn test_debug() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let connection = driver_manager.connect(DATABASE_URL).await?;

        assert!(format!("{:?}", connection).contains("Connection"));
        assert!(format!("{:?}", connection).contains(DATABASE_URL));
        Ok(())
    }

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

                if let Some(Value::I64(id)) = row.get(0) {
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
        match query_result.next().await {
            Some(row) => {
                assert_eq!(row.len(), 5);

                if let Some(Value::String(value)) = row.get(0) {
                    assert_eq!(value, "foo");
                } else {
                    assert!(false);
                }

                if let Some(Value::I64(value)) = row.get(1) {
                    assert_eq!(*value, 123);
                } else {
                    assert!(false);
                }

                if let Some(Value::I64(value)) = row.get(2) {
                    assert_eq!(*value, 456);
                } else {
                    assert!(false);
                }

                if let Some(Value::F64(value)) = row.get(3) {
                    assert_eq!(*value, 789.123);
                } else {
                    assert!(false);
                }

                if let Some(Value::Bytes(value)) = row.get(4) {
                    assert_eq!(*value, vec![42]);
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

    async fn test_data_type(sql: &str) -> anyhow::Result<Option<Value>> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        let mut query_result = connection.query(sql).await?;
        let mut value: Option<Value> = None;

        assert_eq!(query_result.columns().await.len(), 1);

        if let Some(row) = query_result.next().await {
            assert_eq!(row.len(), 1);

            value = row.get(0).cloned();
        }
        assert!(query_result.next().await.is_none());

        connection.close().await?;
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
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;

        let tables = connection.tables().await?;
        assert_eq!(tables, vec!["contacts", "users"]);

        let indexes = connection.indexes(None).await?;
        assert_eq!(
            indexes,
            vec!["sqlite_autoindex_contacts_1", "sqlite_autoindex_users_1"]
        );

        let indexes = connection.indexes(Some("users")).await?;
        assert_eq!(indexes, vec!["sqlite_autoindex_users_1"]);

        connection.close().await?;
        Ok(())
    }
}
