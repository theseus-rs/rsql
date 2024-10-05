use crate::error::Result;
use crate::sqlite::metadata;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, Metadata, QueryResult, StatementMetadata};
use async_trait::async_trait;
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, SQLiteDialect};
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
    url: String,
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
            let file = params.remove("file").unwrap_or_default();
            let query: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(params.iter())
                .finish();
            format!("sqlite://{file}?{query}").to_string()
        };

        let options = SqliteConnectOptions::from_str(database_url.as_str())?
            .auto_vacuum(SqliteAutoVacuum::None)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        let connection = Connection { url, pool };

        Ok(connection)
    }
}

#[async_trait]
impl crate::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let rows = sqlx::query(sql).execute(&self.pool).await?.rows_affected();
        Ok(rows)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
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
                let value = Self::convert_to_value(&row, column)?;
                row_data.push(value);
            }
            rows.push(row_data);
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;
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
    fn convert_to_value(row: &SqliteRow, column: &SqliteColumn) -> Result<Value> {
        let column_name = column.name();
        let column_type = column.type_info();
        let column_type_name = column_type.name();

        match column_type_name {
            "TEXT" => {
                return match row.try_get(column_name)? {
                    Some(v) => Ok(Value::String(v)),
                    None => Ok(Value::Null),
                };
            }
            // Not currently supported by sqlx
            // "NUMERIC" => {
            //     return match row.try_get(column_name)? {
            //         Some(v) => Ok(Value::String(v)),
            //         None => Ok(Value::Null),
            //     };
            // }
            "INTEGER" => {
                return match row.try_get(column_name)? {
                    Some(v) => Ok(Value::I64(v)),
                    None => Ok(Value::Null),
                };
            }
            "REAL" => {
                return match row.try_get(column_name)? {
                    Some(v) => Ok(Value::F64(v)),
                    None => Ok(Value::Null),
                };
            }
            "BLOB" => {
                return match row.try_get(column_name)? {
                    Some(v) => Ok(Value::Bytes(v)),
                    None => Ok(Value::Null),
                };
            }
            _ => {}
        }

        if let Ok(value) = row.try_get::<Option<String>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::String(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<Vec<u8>>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Bytes(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i8>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I8(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i16>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I16(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i32>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I32(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<f32>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::F32(v)),
                None => Ok(Value::Null),
            }
        } else {
            let column_type = column.type_info();
            let type_name = format!("{column_type:?}");

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
        assert_eq!(DATABASE_URL, connection.url());
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
                Value::I8(123),
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
    async fn test_data_type_i8() -> anyhow::Result<()> {
        assert_eq!(test_data_type("SELECT 127").await?, Some(Value::I8(127)));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i16() -> anyhow::Result<()> {
        assert_eq!(
            test_data_type("SELECT 32767").await?,
            Some(Value::I16(32_767))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i32() -> anyhow::Result<()> {
        assert_eq!(
            test_data_type("SELECT 2147483647").await?,
            Some(Value::I32(2_147_483_647))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f32() -> anyhow::Result<()> {
        assert_eq!(
            test_data_type("SELECT 12345.678").await?,
            Some(Value::F32(12_345.678))
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
