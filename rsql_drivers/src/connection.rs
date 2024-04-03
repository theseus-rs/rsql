use crate::error::Result;
use crate::value::Value;
use async_trait::async_trait;
use mockall::predicate::*;
use mockall::*;
use std::fmt::Debug;

/// Results from a query
#[async_trait]
pub trait QueryResult: Debug + Send + Sync {
    async fn columns(&self) -> Vec<String>;
    async fn rows(&self) -> Vec<Vec<Option<Value>>>;
}

/// Query result with a limit
#[derive(Debug)]
pub struct LimitQueryResult {
    inner: Box<dyn QueryResult>,
    limit: usize,
}

impl LimitQueryResult {
    pub fn new(inner: Box<dyn QueryResult>, limit: usize) -> Self {
        Self { inner, limit }
    }
}

#[async_trait]
impl QueryResult for LimitQueryResult {
    async fn columns(&self) -> Vec<String> {
        self.inner.columns().await
    }

    async fn rows(&self) -> Vec<Vec<Option<Value>>> {
        let rows = self.inner.rows().await;

        if rows.len() <= self.limit {
            return rows;
        }

        rows[0..self.limit].to_vec()
    }
}

/// In-memory query result
#[derive(Clone, Debug, Default)]
pub struct MemoryQueryResult {
    columns: Vec<String>,
    rows: Vec<Vec<Option<Value>>>,
}

impl MemoryQueryResult {
    pub fn new(columns: Vec<String>, rows: Vec<Vec<Option<Value>>>) -> Self {
        Self { columns, rows }
    }
}

#[async_trait]
impl QueryResult for MemoryQueryResult {
    async fn columns(&self) -> Vec<String> {
        self.columns.clone()
    }

    async fn rows(&self) -> Vec<Vec<Option<Value>>> {
        self.rows.clone()
    }
}

/// Connection to a database
#[automock]
#[async_trait]
pub trait Connection: Debug + Send + Sync {
    async fn execute(&self, sql: &str) -> Result<u64>;
    async fn indexes<'table>(&mut self, table: Option<&'table str>) -> Result<Vec<String>>;
    async fn query(&self, sql: &str) -> Result<Box<dyn QueryResult>>;
    async fn tables(&mut self) -> Result<Vec<String>>;
    async fn close(&mut self) -> Result<()>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_memory_query_result_new() {
        let columns = vec!["a".to_string()];
        let rows = vec![vec![Some(Value::String("foo".to_string()))]];

        let result = MemoryQueryResult::new(columns, rows);

        let columns = result.columns().await;
        let column = columns.get(0).expect("no column");
        assert_eq!(column, &"a".to_string());

        let rows = result.rows().await;
        let row = rows.get(0).expect("no rows");
        let data = row.get(0).expect("no row data");
        let value = data.as_ref().expect("no value");
        assert_eq!(value, &Value::String("foo".to_string()));
    }

    #[tokio::test]
    async fn test_limit_query_result() {
        let columns = vec!["id".to_string()];
        let rows = vec![
            vec![Some(Value::I64(1))],
            vec![Some(Value::I64(2))],
            vec![Some(Value::I64(3))],
            vec![Some(Value::I64(4))],
            vec![Some(Value::I64(5))],
        ];
        let memory_result = MemoryQueryResult::new(columns, rows);
        let result = LimitQueryResult::new(Box::new(memory_result), 2);

        let columns = result.columns().await;
        let column = columns.get(0).expect("no column");
        assert_eq!(column, &"id".to_string());

        let rows = result.rows().await;
        let data: Vec<String> = rows
            .iter()
            .map(|row| {
                if let Some(value) = row.get(0).expect("no row data") {
                    return value.to_string();
                }
                "0".to_string()
            })
            .collect();

        assert_eq!(data, ["1".to_string(), "2".to_string()]);
    }

    #[tokio::test]
    async fn test_limit_query_result_limit_exceeds_rows() {
        let columns = vec!["id".to_string()];
        let rows = vec![vec![Some(Value::I64(1))]];
        let memory_result = MemoryQueryResult::new(columns, rows);
        let result = LimitQueryResult::new(Box::new(memory_result), 100);

        let columns = result.columns().await;
        let column = columns.get(0).expect("no column");
        assert_eq!(column, &"id".to_string());

        let rows = result.rows().await;
        let data: Vec<String> = rows
            .iter()
            .map(|row| {
                if let Some(value) = row.get(0).expect("no row data") {
                    return value.to_string();
                }
                "0".to_string()
            })
            .collect();

        assert_eq!(data, ["1".to_string()]);
    }
}
