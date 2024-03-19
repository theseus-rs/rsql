use crate::drivers::error::Result;
use crate::drivers::value::Value;
use async_trait::async_trait;
use std::fmt::Debug;

/// Results from a query or execute
#[derive(Debug)]
pub enum Results {
    Query(Box<dyn QueryResult>),
    Execute(u64),
}

/// Results from a query
#[async_trait]
pub trait QueryResult: Debug + Send + Sync {
    async fn columns(&self) -> Vec<String>;
    async fn rows(&self) -> Vec<Vec<Option<Value>>>;
}

/// In-memory query result
#[derive(Debug, Default)]
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
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Connection: Debug + Send {
    async fn execute(&self, sql: &str) -> Result<Results>;
    async fn query(&self, sql: &str) -> Result<Results>;
    async fn tables(&mut self) -> Result<Vec<String>>;
    async fn stop(&mut self) -> Result<()>;
}
