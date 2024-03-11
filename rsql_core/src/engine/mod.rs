#[cfg(feature = "postgresql")]
pub(crate) mod postgresql;

#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;

use anyhow::{bail, Result};
use async_trait::async_trait;
use sqlx::any::install_default_drivers;

pub(crate) struct QueryResult {
    pub(crate) columns: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
}

#[async_trait]
pub trait Engine: Send {
    async fn execute(&self, sql: &str) -> Result<u64>;
    async fn query(&self, sql: &str) -> Result<QueryResult>;
    async fn tables(&mut self) -> Result<Vec<String>>;
    async fn stop(&mut self) -> Result<()>;
}

/// url = "postgresql::embdeded:"
/// url = "sqlite::memory:"
pub async fn load(url: &str) -> Result<Box<dyn Engine>> {
    install_default_drivers();

    let database = match url.split_once(':') {
        Some((before, _)) => before,
        None => "",
    };

    match database {
        #[cfg(feature = "postgresql")]
        "postgresql" => Ok(Box::new(postgresql::Engine::new(url).await?)),
        #[cfg(feature = "sqlite")]
        "sqlite" => Ok(Box::new(sqlite::Engine::new(url).await?)),
        _ => bail!("Invalid database url: {url}"),
    }
}
