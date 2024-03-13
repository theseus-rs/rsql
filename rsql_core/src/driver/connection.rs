use crate::driver::value::Value;
use async_trait::async_trait;

pub struct QueryResult {
    pub(crate) columns: Vec<String>,
    pub(crate) rows: Vec<Vec<Option<Value>>>,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Connection: Send {
    async fn execute(&self, sql: &str) -> anyhow::Result<u64>;
    async fn query(&self, sql: &str) -> anyhow::Result<QueryResult>;
    async fn tables(&mut self) -> anyhow::Result<Vec<String>>;
    async fn stop(&mut self) -> anyhow::Result<()>;
}
