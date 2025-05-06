use crate::metadata;
use crate::value::ToValue;
use async_trait::async_trait;
use polars_sql::SQLContext;
use rsql_driver::Error::{ConversionError, InvalidUrl, IoError};
use rsql_driver::{MemoryQueryResult, Metadata, QueryResult, Result};
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Connection for drivers based on Polars `SQLContext`
pub struct Connection {
    url: String,
    context: Arc<Mutex<SQLContext>>,
}

impl Connection {
    /// Create a new `Connection`
    ///
    /// # Errors
    /// if the URL is invalid
    #[expect(clippy::unused_async)]
    pub async fn new(url: &str, context: SQLContext) -> Result<Self> {
        Ok(Self {
            url: url.to_string(),
            context: Arc::new(Mutex::new(context)),
        })
    }
}

impl Connection {
    /// Get the `SQLContext`
    pub(crate) fn context(&self) -> Arc<Mutex<SQLContext>> {
        self.context.clone()
    }
}

/// Get the table name from the file name
///
/// # Errors
/// if the file name is invalid
pub fn get_table_name<S: AsRef<str>>(file_name: S) -> Result<String> {
    let file_name = file_name.as_ref();
    let file_name = Path::new(file_name)
        .file_name()
        .ok_or(InvalidUrl("Invalid file name".to_string()))?
        .to_str()
        .ok_or(InvalidUrl("Invalid file name".to_string()))?;
    let table_name = file_name.split('.').next().unwrap_or(file_name);
    Ok(table_name.to_string())
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let mut context = self.context.lock().await;
        let result = context
            .execute(sql)
            .map_err(|error| IoError(error.to_string()))?;
        let data_frame = result
            .collect()
            .map_err(|error| IoError(error.to_string()))?;
        let rows =
            u64::try_from(data_frame.height()).map_err(|error| IoError(error.to_string()))?;
        Ok(rows)
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let mut context = self.context.lock().await;
        let result = context
            .execute(sql)
            .map_err(|error| IoError(error.to_string()))?;
        let data_frame = result
            .collect()
            .map_err(|error| IoError(error.to_string()))?;
        let columns = data_frame
            .get_column_names()
            .iter()
            .map(ToString::to_string)
            .collect();
        let mut rows = Vec::new();

        // Convert the data frame to a vector of rows
        for data_frame_row in data_frame.iter() {
            for (row, data) in data_frame_row.iter().enumerate() {
                let row = if let Some(row) = rows.get_mut(row) {
                    row
                } else {
                    let row = Vec::new();
                    rows.push(row);
                    rows.last_mut().ok_or(ConversionError(
                        "Failed to convert DataFrame to QueryResult".to_string(),
                    ))?
                };
                let value = data.to_value();
                row.push(value);
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }
}

#[expect(clippy::missing_fields_in_debug)]
impl Debug for Connection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("url", &self.url)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use polars::prelude::*;
    use rsql_driver::{Connection, Value};

    #[tokio::test]
    async fn test_connection() -> Result<()> {
        let ids = Series::new("id".into(), &[1i64, 2i64]);
        let names = Series::new("name".into(), &["John Doe", "Jane Smith"]);
        let data_frame = DataFrame::new(vec![Column::from(ids), Column::from(names)])
            .map_err(|error| IoError(error.to_string()))?;
        let mut context = SQLContext::new();
        context.register("users", data_frame.lazy());
        let mut connection = super::Connection::new("polars://", context).await?;

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
}
