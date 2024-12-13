use crate::error::Result;
use crate::polars::metadata;
use crate::Error::ConversionError;
use crate::{MemoryQueryResult, Metadata, QueryResult, Value};
use async_trait::async_trait;
use polars::prelude::AnyValue;
use polars_sql::SQLContext;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Connection for drivers based on Polars `SQLContext`
pub struct Connection {
    url: String,
    context: Arc<Mutex<SQLContext>>,
}

impl Connection {
    #[expect(clippy::unused_async)]
    pub async fn new(url: String, context: SQLContext) -> Result<Self> {
        Ok(Self {
            url,
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

#[async_trait]
impl crate::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let mut context = self.context.lock().await;
        let result = context.execute(sql)?;
        let data_frame = result.collect()?;
        let rows = u64::try_from(data_frame.height())?;
        Ok(rows)
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let mut context = self.context.lock().await;
        let result = context.execute(sql)?;
        let data_frame = result.collect()?;
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
                let value = match data {
                    AnyValue::Null => Value::Null,
                    AnyValue::Boolean(v) => Value::Bool(v),
                    AnyValue::Binary(v) => Value::Bytes(v.to_vec()),
                    AnyValue::Float32(v) => Value::F32(v),
                    AnyValue::Float64(v) => Value::F64(v),
                    AnyValue::Int8(v) => Value::I8(v),
                    AnyValue::Int16(v) => Value::I16(v),
                    AnyValue::Int32(v) => Value::I32(v),
                    AnyValue::Int64(v) => Value::I64(v),
                    AnyValue::String(v) => Value::String(v.to_string()),
                    AnyValue::UInt8(v) => Value::U8(v),
                    AnyValue::UInt16(v) => Value::U16(v),
                    AnyValue::UInt32(v) => Value::U32(v),
                    AnyValue::UInt64(v) => Value::U64(v),
                    _ => Value::String(data.to_string()),
                };
                row.push(value);
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
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
    use crate::{Connection, Value};
    use polars::prelude::*;

    #[tokio::test]
    async fn test_connection() -> anyhow::Result<()> {
        let ids = Series::new("id".into(), &[1i64, 2i64]);
        let names = Series::new("name".into(), &["John Doe", "Jane Smith"]);
        let data_frame = DataFrame::new(vec![Column::from(ids), Column::from(names)])?;
        let mut context = SQLContext::new();
        context.register("users", data_frame.lazy());
        let mut connection = super::Connection::new("polars://".to_string(), context).await?;

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
