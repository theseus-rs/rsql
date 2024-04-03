use crate::error::Result;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, QueryResult};
use async_trait::async_trait;
use indoc::indoc;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteColumn, SqliteConnectOptions, SqliteRow};
use sqlx::{Column, Row, SqlitePool, TypeInfo};
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "sqlite"
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
    pool: SqlitePool,
}

impl Connection {
    pub(crate) async fn new(url: String) -> Result<Connection> {
        let parsed_url = Url::parse(url.as_str())?;
        let mut params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let memory = params
            .remove("memory")
            .map_or(false, |value| value == "true");

        let database_url = if memory {
            "sqlite::memory:".to_string()
        } else {
            let file = params.remove("file").unwrap_or("".to_string());
            let query: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(params.iter())
                .finish();
            format!("sqlite://{file}?{query}").to_string()
        };

        let options = SqliteConnectOptions::from_str(database_url.as_str())?
            .auto_vacuum(SqliteAutoVacuum::None)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
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
            SELECT name
              FROM sqlite_master
             WHERE type = 'index'
        "#}
        .to_string();
        if table.is_some() {
            sql = format!("{sql} AND tbl_name = $1");
        }
        sql = format!("{sql} ORDER BY name");
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
        let sql = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
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
    fn convert_to_value(&self, row: &SqliteRow, column: &SqliteColumn) -> Result<Option<Value>> {
        let column_name = column.name();
        let column_type = column.type_info();
        let column_type_name = column_type.name();

        match column_type_name {
            "TEXT" => {
                let value: Option<String> = row.try_get(column_name)?;
                return Ok(value.map(Value::String));
            }
            // Not currently supported by sqlx
            // "NUMERIC" => {
            //     let value: Option<String> = row.try_get(column_name)?;
            //     return Ok(value.map(Value::String));
            // }
            "INTEGER" => {
                let value: Option<i64> = row.try_get(column_name)?;
                return Ok(value.map(Value::I64));
            }
            "REAL" => {
                let value: Option<f64> = row.try_get(column_name)?;
                return Ok(value.map(Value::F64));
            }
            "BLOB" => {
                let value: Option<Vec<u8>> = row.try_get(column_name)?;
                return Ok(value.map(Value::Bytes));
            }
            _ => {}
        }

        if let Ok(value) = row.try_get(column_name) {
            let value: Option<String> = value;
            Ok(value.map(Value::String))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<Vec<u8>> = value;
            Ok(value.map(Value::Bytes))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i8> = value;
            Ok(value.map(Value::I8))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i16> = value;
            Ok(value.map(Value::I16))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i32> = value;
            Ok(value.map(Value::I32))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<f32> = value;
            Ok(value.map(Value::F32))
        } else {
            let column_type = column.type_info();
            let type_name = format!("{:?}", column_type);

            Err(UnsupportedColumnType {
                column_name: column_name.to_string(),
                column_type: type_name,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{DriverManager, Value};

    const DATABASE_URL: &str = "sqlite://?memory=true";

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_limit_rows() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let connection = driver_manager.connect(DATABASE_URL).await?;
        let query_result = connection.query("SELECT 1 UNION ALL SELECT 2", 1).await?;
        assert_eq!(query_result.rows().await.len(), 1);
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

        let query_result = connection.query("SELECT id, name FROM person", 0).await?;
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

        let query_result = connection
            .query("SELECT t, nu, i, r, no FROM t1", 0)
            .await?;
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

                if let Some(Value::I8(value)) = &row[1] {
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

        connection.close().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> anyhow::Result<Option<Value>> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        let query_result = connection.query(sql, 0).await?;
        let mut value: Option<Value> = None;

        assert_eq!(query_result.columns().await.len(), 1);
        assert_eq!(query_result.rows().await.len(), 1);

        if let Some(row) = query_result.rows().await.get(0) {
            assert_eq!(row.len(), 1);

            value = row[0].clone();
        }

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
    async fn test_data_type_i8() -> anyhow::Result<()> {
        match test_data_type("SELECT 127").await? {
            Some(value) => assert_eq!(value, Value::I8(127)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i16() -> anyhow::Result<()> {
        match test_data_type("SELECT 32767").await? {
            Some(value) => assert_eq!(value, Value::I16(32_767)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i32() -> anyhow::Result<()> {
        match test_data_type("SELECT 2147483647").await? {
            Some(value) => assert_eq!(value, Value::I32(2_147_483_647)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f32() -> anyhow::Result<()> {
        match test_data_type("SELECT 12345.67890").await? {
            Some(value) => assert_eq!(value, Value::F32(12_345.67890)),
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

        let indexes = connection.indexes(None).await?;
        assert_eq!(
            indexes,
            vec!["sqlite_autoindex_contacts_1", "sqlite_autoindex_users_1"]
        );

        let indexes = connection.indexes(Some("users")).await?;
        assert_eq!(indexes, vec!["sqlite_autoindex_users_1"]);

        let tables = connection.tables().await?;
        assert_eq!(tables, vec!["contacts", "users"]);

        connection.close().await?;
        Ok(())
    }
}