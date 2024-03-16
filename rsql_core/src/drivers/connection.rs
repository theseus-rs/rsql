use crate::drivers::value::Value;
use async_trait::async_trait;
use std::fmt::Debug;

/// Results from a query or execute
#[derive(Debug)]
pub enum Results {
    Query(QueryResult),
    Execute(u64),
}

/// Results from a query
#[derive(Debug, Default)]
pub struct QueryResult {
    pub(crate) columns: Vec<String>,
    pub(crate) rows: Vec<Vec<Option<Value>>>,
}

/// Connection to a database
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Connection: Debug + Send {
    async fn execute(&self, sql: &str) -> anyhow::Result<Results>;
    async fn query(&self, sql: &str) -> anyhow::Result<Results>;
    async fn tables(&mut self) -> anyhow::Result<Vec<String>>;
    async fn stop(&mut self) -> anyhow::Result<()>;
}
