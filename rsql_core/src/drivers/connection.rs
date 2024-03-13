use crate::drivers::value::Value;
use async_trait::async_trait;

pub enum Results {
    Query(QueryResult),
    Execute(u64),
}
pub struct QueryResult {
    pub(crate) columns: Vec<String>,
    pub(crate) rows: Vec<Vec<Option<Value>>>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Connection: Send {
    async fn execute(&self, sql: &str) -> anyhow::Result<Results>;
    async fn query(&self, sql: &str) -> anyhow::Result<Results>;
    async fn tables(&mut self) -> anyhow::Result<Vec<String>>;
    async fn stop(&mut self) -> anyhow::Result<()>;
}
