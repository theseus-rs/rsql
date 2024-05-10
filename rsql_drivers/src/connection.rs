use crate::error::Result;
use crate::row::Row;
use crate::Metadata;
use async_trait::async_trait;
use mockall::predicate::*;
use mockall::*;
use std::fmt::Debug;

/// Results from a query
#[async_trait]
pub trait QueryResult: Debug + Send + Sync {
    async fn columns(&self) -> Vec<String>;
    async fn next(&mut self) -> Option<Row>;
}

/// Query result with a limit
#[derive(Debug)]
pub struct LimitQueryResult {
    inner: Box<dyn QueryResult>,
    row_index: usize,
    limit: usize,
}

impl LimitQueryResult {
    pub fn new(inner: Box<dyn QueryResult>, limit: usize) -> Self {
        Self {
            inner,
            row_index: 0,
            limit,
        }
    }
}

#[async_trait]
impl QueryResult for LimitQueryResult {
    async fn columns(&self) -> Vec<String> {
        self.inner.columns().await
    }

    async fn next(&mut self) -> Option<Row> {
        if self.row_index >= self.limit {
            return None;
        }

        let value = self.inner.next().await;
        self.row_index += 1;
        value
    }
}

/// In-memory query result
#[derive(Clone, Debug, Default)]
pub struct MemoryQueryResult {
    columns: Vec<String>,
    row_index: usize,
    rows: Vec<Row>,
}

impl MemoryQueryResult {
    pub fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self {
            columns,
            row_index: 0,
            rows,
        }
    }
}

#[async_trait]
impl QueryResult for MemoryQueryResult {
    async fn columns(&self) -> Vec<String> {
        self.columns.clone()
    }

    async fn next(&mut self) -> Option<Row> {
        let result = self.rows.get(self.row_index).cloned();
        self.row_index += 1;
        result
    }
}

/// Connection to a database
#[automock]
#[async_trait]
pub trait Connection: Debug + Send + Sync {
    async fn execute(&mut self, sql: &str) -> Result<u64>;
    async fn metadata(&mut self) -> Result<Metadata> {
        unimplemented!()
    }
    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>>;
    async fn close(&mut self) -> Result<()>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Value;

    #[tokio::test]
    async fn test_memory_query_result_new() {
        let columns = vec!["a".to_string()];
        let rows = vec![Row::new(vec![Value::String("foo".to_string())])];

        let mut result = MemoryQueryResult::new(columns, rows);

        let columns = result.columns().await;
        let column = columns.get(0).expect("no column");
        assert_eq!(column, &"a".to_string());

        let row = result.next().await.expect("no row");
        let value = row.first().expect("no value");
        assert_eq!(value, &Value::String("foo".to_string()));
    }

    #[tokio::test]
    async fn test_limit_query_result() {
        let columns = vec!["id".to_string()];
        let rows = vec![
            Row::new(vec![Value::I64(1)]),
            Row::new(vec![Value::I64(2)]),
            Row::new(vec![Value::I64(3)]),
            Row::new(vec![Value::I64(4)]),
            Row::new(vec![Value::I64(5)]),
        ];
        let memory_result = MemoryQueryResult::new(columns, rows);
        let mut result = LimitQueryResult::new(Box::new(memory_result), 2);

        let columns = result.columns().await;
        let column = columns.get(0).expect("no column");
        assert_eq!(column, &"id".to_string());

        let mut data: Vec<String> = Vec::new();
        while let Some(row) = result.next().await {
            let value = row.first().expect("no value");
            data.push(value.to_string());
        }

        assert_eq!(data, ["1".to_string(), "2".to_string()]);
    }

    #[tokio::test]
    async fn test_limit_query_result_limit_exceeds_rows() {
        let columns = vec!["id".to_string()];
        let rows = vec![Row::new(vec![Value::I64(1)])];
        let memory_result = MemoryQueryResult::new(columns, rows);
        let mut result = LimitQueryResult::new(Box::new(memory_result), 100);

        let columns = result.columns().await;
        let column = columns.get(0).expect("no column");
        assert_eq!(column, &"id".to_string());

        let mut data: Vec<String> = Vec::new();
        while let Some(row) = result.next().await {
            let value = row.first().expect("no value");
            data.push(value.to_string());
        }

        assert_eq!(data, ["1".to_string()]);
    }
}
