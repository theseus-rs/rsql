use crate::configuration::Configuration;
use crate::drivers::connection::{QueryResult, Results};
use crate::drivers::value::Value;
use anyhow::{bail, Result};
use async_trait::async_trait;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteColumn, SqliteConnectOptions, SqliteRow};
use sqlx::{Column, Row, SqlitePool, TypeInfo};
use std::str::FromStr;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::drivers::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "sqlite"
    }

    async fn connect(
        &self,
        configuration: &Configuration,
        url: &str,
    ) -> Result<Box<dyn crate::drivers::Connection>> {
        let connection = Connection::new(configuration, url).await?;
        Ok(Box::new(connection))
    }
}

#[derive(Debug)]
pub(crate) struct Connection {
    pool: SqlitePool,
}

impl Connection {
    pub(crate) async fn new(_configuration: &Configuration, url: &str) -> Result<Connection> {
        let options = SqliteConnectOptions::from_str(url)?
            .auto_vacuum(SqliteAutoVacuum::None)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        let connection = Connection { pool };

        Ok(connection)
    }
}

#[async_trait]
impl crate::drivers::Connection for Connection {
    async fn execute(&self, sql: &str) -> Result<Results> {
        let rows = sqlx::query(sql).execute(&self.pool).await?.rows_affected();
        Ok(Results::Execute(rows))
    }

    async fn query(&self, sql: &str) -> Result<Results> {
        let query_rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let columns = if let Some(row) = query_rows.first() {
            row.columns()
                .iter()
                .map(|column| column.name().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let mut rows = Vec::new();
        for row in query_rows {
            let mut row_data = Vec::new();
            for column in row.columns() {
                let value = self.convert_to_value(&row, column)?;
                row_data.push(value);
            }
            rows.push(row_data);
        }

        let query_result = QueryResult { columns, rows };
        Ok(Results::Query(query_result))
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let mut tables = Vec::new();

        for row in rows {
            match row.try_get::<String, _>(0) {
                Ok(table_name) => tables.push(table_name),
                Err(error) => bail!("Error: {:?}", error),
            }
        }

        Ok(tables)
    }

    async fn stop(&mut self) -> Result<()> {
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
            bail!(
                "column type [{:?}] not supported for column [{}]",
                column_type,
                column_name
            );
        }
    }
}

#[cfg(test)]
mod test {
    use crate::configuration::Configuration;
    use crate::drivers::{DriverManager, Results, Value};
    use anyhow::Result;

    const DATABASE_URL: &str = "sqlite::memory:";

    #[tokio::test]
    async fn test_driver_connect() -> Result<()> {
        let configuration = Configuration::default();
        let drivers = DriverManager::default();
        let mut connection = drivers.connect(&configuration, DATABASE_URL).await?;
        connection.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> Result<()> {
        let configuration = &Configuration::default();
        let drivers = DriverManager::default();
        let mut connection = drivers.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE person (id INTEGER, name TEXT)")
            .await?;

        let execute_results = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        if let Results::Execute(rows) = execute_results {
            assert_eq!(rows, 1);
        }

        let results = connection.query("SELECT id, name FROM person").await?;
        if let Results::Query(query_result) = results {
            assert_eq!(query_result.columns, vec!["id", "name"]);
            assert_eq!(query_result.rows.len(), 1);
            match query_result.rows.get(0) {
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

        let tables = connection.tables().await?;
        assert_eq!(tables, vec!["person"]);

        connection.stop().await?;
        Ok(())
    }

    /// Ref: https://www.sqlite.org/datatype3.html
    #[tokio::test]
    async fn test_table_data_types() -> Result<()> {
        let configuration = &Configuration::default();
        let drivers = DriverManager::default();
        let mut connection = drivers.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE t1(t TEXT, nu NUMERIC, i INTEGER, r REAL, no BLOB)")
            .await?;

        let execute_results = connection
            .execute("INSERT INTO t1 (t, nu, i, r, no) VALUES ('foo', 123, 456, 789.123, x'2a')")
            .await?;
        if let Results::Execute(rows) = execute_results {
            assert_eq!(rows, 1);
        }

        let results = connection.query("SELECT t, nu, i, r, no FROM t1").await?;
        if let Results::Query(query_result) = results {
            assert_eq!(query_result.columns, vec!["t", "nu", "i", "r", "no"]);
            assert_eq!(query_result.rows.len(), 1);
            match query_result.rows.get(0) {
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
        }

        connection.stop().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> Result<Option<Value>> {
        let configuration = Configuration::default();
        let drivers = DriverManager::default();
        let mut connection = drivers.connect(&configuration, DATABASE_URL).await?;

        let results = connection.query(sql).await?;
        let mut value: Option<Value> = None;

        if let Results::Query(query_result) = results {
            assert_eq!(query_result.columns.len(), 1);
            assert_eq!(query_result.rows.len(), 1);

            if let Some(row) = query_result.rows.get(0) {
                assert_eq!(row.len(), 1);

                value = row[0].clone();
            }
        }

        connection.stop().await?;
        Ok(value)
    }

    #[tokio::test]
    async fn test_data_type_bytes() -> Result<()> {
        match test_data_type("SELECT x'2a'").await? {
            Some(value) => assert_eq!(value, Value::Bytes(vec![42])),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i8() -> Result<()> {
        match test_data_type("SELECT 127").await? {
            Some(value) => assert_eq!(value, Value::I8(127)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i16() -> Result<()> {
        match test_data_type("SELECT 32767").await? {
            Some(value) => assert_eq!(value, Value::I16(32_767)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i32() -> Result<()> {
        match test_data_type("SELECT 2147483647").await? {
            Some(value) => assert_eq!(value, Value::I32(2_147_483_647)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f32() -> Result<()> {
        match test_data_type("SELECT 12345.67890").await? {
            Some(value) => assert_eq!(value, Value::F32(12_345.67890)),
            _ => assert!(false),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_string() -> Result<()> {
        match test_data_type("SELECT 'foo'").await? {
            Some(value) => assert_eq!(value, Value::String("foo".to_string())),
            _ => assert!(false),
        }
        Ok(())
    }
}
