#[cfg(feature = "postgresql")]
pub(crate) mod postgresql;

mod drivers;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;
mod value;

pub use drivers::{Driver, DriverManager};

use crate::engine::value::Value;
use anyhow::Result;
use async_trait::async_trait;
#[cfg(test)]
use mockall::{automock, predicate::*};

pub struct QueryResult {
    pub(crate) columns: Vec<String>,
    pub(crate) rows: Vec<Vec<Option<Value>>>,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait Engine: Send {
    async fn execute(&self, sql: &str) -> Result<u64>;
    async fn query(&self, sql: &str) -> Result<QueryResult>;
    async fn tables(&mut self) -> Result<Vec<String>>;
    async fn stop(&mut self) -> Result<()>;
}
