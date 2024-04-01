use crate::error::Result;
use crate::value::Value;
use async_trait::async_trait;
use mockall::predicate::*;
use mockall::*;
use std::fmt::Debug;

/// Results from a query or execute
#[derive(Debug)]
pub enum Results {
    Query(Box<dyn QueryResult>),
    Execute(u64),
}

impl Results {
    pub fn is_query(&self) -> bool {
        matches!(self, Results::Query(_))
    }

    pub fn is_execute(&self) -> bool {
        matches!(self, Results::Execute(_))
    }
}

/// Results from a query
#[async_trait]
pub trait QueryResult: Debug + Send + Sync {
    async fn columns(&self) -> Vec<String>;
    async fn rows(&self) -> Vec<Vec<Option<Value>>>;
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
    async fn execute(&self, sql: &str) -> Result<Results>;
    async fn indexes<'table>(&mut self, table: Option<&'table str>) -> Result<Vec<String>>;
    async fn query(&self, sql: &str, limit: u64) -> Result<Results>;
    async fn tables(&mut self) -> Result<Vec<String>>;
    async fn stop(&mut self) -> Result<()>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_results_is_query() {
        let query_results = Box::new(MemoryQueryResult::default());
        assert!(Results::Query(query_results).is_query());
    }

    #[test]
    fn test_results_is_execute() {
        assert!(Results::Execute(42).is_execute());
    }

    #[test]
    fn test_memory_query_result_new() {
        let columns = vec!["a".to_string()];
        let rows = vec![vec![Some(Value::String("foo".to_string()))]];

        let result = MemoryQueryResult::new(columns, rows);

        let column = result.columns.get(0).expect("no column");
        assert_eq!(column, &"a".to_string());
        let row = result.rows.get(0).expect("no rows");
        let data = row.get(0).expect("no row data");
        let value = data.as_ref().expect("no value");
        assert_eq!(value, &Value::String("foo".to_string()));
    }
}
