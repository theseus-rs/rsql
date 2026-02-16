use crate::metadata;
use crate::results::SqliteQueryResult;
use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Error::IoError;
use rsql_driver::{Metadata, QueryResult, Result, StatementMetadata, ToSql, UrlExtension, Value};
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, SQLiteDialect};
use sqlx::sqlite::{SqliteArguments, SqliteAutoVacuum, SqliteConnectOptions};
use sqlx::{Column, Row, Sqlite, SqlitePool};
use std::collections::HashMap;
use std::str::FromStr;
use url::{Url, form_urlencoded};

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "sqlite"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let connection = Connection::new(url).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        let media_types = file_type.media_types();
        media_types.contains(&"application/vnd.sqlite3")
            || media_types.contains(&"application/x-sqlite3")
    }
}

#[derive(Debug)]
pub struct Connection {
    url: String,
    pool: SqlitePool,
}

impl Connection {
    pub(crate) async fn new(url: &str) -> Result<Connection> {
        let parsed_url = Url::parse(url)?;
        let database_url = if let Ok(file_name) = parsed_url.to_file() {
            let file_name = file_name.to_string_lossy();
            let params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
            let query: String = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(params.iter())
                .finish();
            format!("sqlite://{file_name}?{query}").to_string()
        } else {
            "sqlite::memory:".to_string()
        };

        let options = SqliteConnectOptions::from_str(database_url.as_str())
            .map_err(|error| IoError(error.to_string()))?
            .auto_vacuum(SqliteAutoVacuum::None)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let connection = Connection {
            url: url.to_string(),
            pool,
        };

        Ok(connection)
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let values = rsql_driver::to_values(params);
        let mut query = sqlx::query(sql);
        for value in &values {
            query = bind_sqlite_value(query, value);
        }
        let rows = query
            .execute(&self.pool)
            .await
            .map_err(|error| IoError(error.to_string()))?
            .rows_affected();
        Ok(rows)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let values = rsql_driver::to_values(params);
        let mut query = sqlx::query(sql);
        for value in &values {
            query = bind_sqlite_value(query, value);
        }
        let query_rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let columns: Vec<String> = query_rows
            .first()
            .map(|row| {
                row.columns()
                    .iter()
                    .map(|column| column.name().to_string())
                    .collect()
            })
            .unwrap_or_default();

        let query_result = SqliteQueryResult::new(columns, query_rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
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

fn bind_sqlite_value<'q>(
    query: sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>>,
    value: &'q Value,
) -> sqlx::query::Query<'q, Sqlite, SqliteArguments<'q>> {
    match value {
        Value::Null => query.bind(None::<String>),
        Value::Bool(v) => query.bind(*v),
        Value::I8(v) => query.bind(i32::from(*v)),
        Value::I16(v) => query.bind(i32::from(*v)),
        Value::I32(v) => query.bind(*v),
        Value::I64(v) => query.bind(*v),
        Value::U8(v) => query.bind(i32::from(*v)),
        Value::U16(v) => query.bind(i32::from(*v)),
        Value::U32(v) => query.bind(i64::from(*v)),
        Value::U64(v) => query.bind(*v as i64),
        Value::F32(v) => query.bind(f64::from(*v)),
        Value::F64(v) => query.bind(*v),
        Value::String(v) => query.bind(v.as_str()),
        Value::Bytes(v) => query.bind(v.as_slice()),
        _ => query.bind(value.to_string()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::Driver;
    use rsql_driver_test_utils::dataset_url;

    const DATABASE_URL: &str = "sqlite://";

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
        let database_url = dataset_url("sqlite", "users.sqlite3");
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
    async fn test_data_type_i8() -> Result<()> {
        assert_eq!(test_data_type("SELECT 127").await?, Some(Value::I8(127)));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i16() -> Result<()> {
        assert_eq!(
            test_data_type("SELECT 32767").await?,
            Some(Value::I16(32_767))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i32() -> Result<()> {
        assert_eq!(
            test_data_type("SELECT 2147483647").await?,
            Some(Value::I32(2_147_483_647))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f32() -> Result<()> {
        assert_eq!(
            test_data_type("SELECT 12345.678").await?,
            Some(Value::F32(12_345.678))
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
                "CREATE TABLE test_params (id INTEGER, name TEXT, score REAL, data BLOB)",
                &[],
            )
            .await?;

        let rows = connection
            .execute(
                "INSERT INTO test_params (id, name, score, data) VALUES (?, ?, ?, ?)",
                &[&1i64, &"Alice", &95.5f64, &vec![1u8, 2, 3] as &dyn ToSql],
            )
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection
            .query(
                "SELECT id, name, score, data FROM test_params WHERE id = ?",
                &[&1i64],
            )
            .await?;

        assert_eq!(query_result.columns(), vec!["id", "name", "score", "data"]);
        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![
                Value::I64(1),
                Value::String("Alice".to_string()),
                Value::F64(95.5),
                Value::Bytes(vec![1, 2, 3]),
            ])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_query_with_string_param() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE test_str (id INTEGER, name TEXT)", &[])
            .await?;

        let _ = connection
            .execute(
                "INSERT INTO test_str (id, name) VALUES (?, ?)",
                &[&1i64, &"Bob"],
            )
            .await?;
        let _ = connection
            .execute(
                "INSERT INTO test_str (id, name) VALUES (?, ?)",
                &[&2i64, &"Alice"],
            )
            .await?;

        let mut query_result = connection
            .query("SELECT id, name FROM test_str WHERE name = ?", &[&"Alice"])
            .await?;

        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![Value::I64(2), Value::String("Alice".to_string())])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }
}
