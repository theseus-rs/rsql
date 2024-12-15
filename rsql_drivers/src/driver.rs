use crate::connection::CachedMetadataConnection;
use crate::error::Result;
use crate::Connection;
use crate::Error::DriverNotFound;
use async_trait::async_trait;
use mockall::automock;
use mockall::predicate::str;
use std::collections::BTreeMap;
use std::fmt::Debug;
use tracing::instrument;
use url::Url;

#[automock]
#[async_trait]
pub trait Driver: Debug + Send + Sync {
    fn identifier(&self) -> &'static str;
    async fn connect(&self, url: String, password: Option<String>) -> Result<Box<dyn Connection>>;
}

/// Manages available drivers
#[derive(Debug)]
pub struct DriverManager {
    drivers: BTreeMap<&'static str, Box<dyn Driver>>,
}

impl DriverManager {
    /// Create a new instance of the `DriverManager`
    #[must_use]
    pub fn new() -> Self {
        DriverManager {
            drivers: BTreeMap::new(),
        }
    }

    /// Add a new driver to the list of available drivers
    pub fn add(&mut self, driver: Box<dyn Driver>) {
        let identifier = driver.identifier();
        let _ = &self.drivers.insert(identifier, driver);
    }

    /// Get a drivers by name
    #[must_use]
    pub fn get(&self, identifier: &str) -> Option<&dyn Driver> {
        self.drivers.get(identifier).map(AsRef::as_ref)
    }

    /// Get an iterator over the available drivers
    pub fn iter(&self) -> impl Iterator<Item = &dyn Driver> {
        self.drivers.values().map(AsRef::as_ref)
    }

    /// Connect to a database
    #[instrument(name = "connect", level = "info", skip(url))]
    pub async fn connect(&self, url: &str) -> Result<Box<dyn Connection>> {
        let parsed_url = Url::parse(url)?;
        let scheme = parsed_url.scheme();
        let password = parsed_url.password().map(ToString::to_string);
        let url = url.to_string();

        match &self.get(scheme) {
            Some(driver) => {
                let connection = driver.connect(url, password).await?;
                let connection = Box::new(CachedMetadataConnection::new(connection));
                Ok(connection)
            }
            None => Err(DriverNotFound {
                identifier: scheme.to_string(),
            }),
        }
    }
}

/// Default implementation for the `DriverManager`
impl Default for DriverManager {
    fn default() -> Self {
        let mut drivers = DriverManager::new();

        #[cfg(any(feature = "mysql", feature = "postgresql", feature = "sqlite"))]
        sqlx::any::install_default_drivers();

        #[cfg(feature = "arrow")]
        drivers.add(Box::new(crate::arrow::Driver));
        #[cfg(feature = "cockroachdb")]
        drivers.add(Box::new(crate::cockroachdb::Driver));
        #[cfg(feature = "csv")]
        drivers.add(Box::new(crate::csv::Driver));
        #[cfg(feature = "delimited")]
        drivers.add(Box::new(crate::delimited::Driver));
        #[cfg(feature = "duckdb")]
        drivers.add(Box::new(crate::duckdb::Driver));
        #[cfg(feature = "json")]
        drivers.add(Box::new(crate::json::Driver));
        #[cfg(feature = "jsonl")]
        drivers.add(Box::new(crate::jsonl::Driver));
        #[cfg(feature = "libsql")]
        drivers.add(Box::new(crate::libsql::Driver));
        #[cfg(feature = "mariadb")]
        drivers.add(Box::new(crate::mariadb::Driver));
        #[cfg(feature = "mysql")]
        drivers.add(Box::new(crate::mysql::Driver));
        #[cfg(feature = "parquet")]
        drivers.add(Box::new(crate::parquet::Driver));
        #[cfg(feature = "postgres")]
        drivers.add(Box::new(crate::postgres::Driver));
        #[cfg(feature = "postgresql")]
        drivers.add(Box::new(crate::postgresql::Driver));
        #[cfg(feature = "redshift")]
        drivers.add(Box::new(crate::redshift::Driver));
        #[cfg(feature = "rusqlite")]
        drivers.add(Box::new(crate::rusqlite::Driver));
        #[cfg(feature = "snowflake")]
        drivers.add(Box::new(crate::snowflake::Driver));
        #[cfg(feature = "sqlite")]
        drivers.add(Box::new(crate::sqlite::Driver));
        #[cfg(feature = "sqlserver")]
        drivers.add(Box::new(crate::sqlserver::Driver));
        #[cfg(feature = "tsv")]
        drivers.add(Box::new(crate::tsv::Driver));

        drivers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockConnection;

    #[test]
    fn test_driver_manager() {
        let identifier = "test";
        let mut mock_driver = MockDriver::new();
        mock_driver.expect_identifier().returning(|| identifier);

        let mut driver_manager = DriverManager::new();
        assert_eq!(driver_manager.drivers.len(), 0);

        driver_manager.add(Box::new(mock_driver));

        assert_eq!(driver_manager.drivers.len(), 1);
        let result = driver_manager.get(identifier);
        assert!(result.is_some());

        let mut driver_count = 0;
        driver_manager.iter().for_each(|_command| {
            driver_count += 1;
        });
        assert_eq!(driver_count, 1);
    }

    #[test]
    fn test_driver_manager_default() {
        let driver_manager = DriverManager::default();
        let driver_count = 0;

        #[cfg(feature = "arrow")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "cockroachdb")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "csv")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "delimited")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "duckdb")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "json")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "jsonl")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "libsql")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "mariadb")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "mysql")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "parquet")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "postgres")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "postgresql")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "redshift")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "rusqlite")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "snowflake")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "sqlite")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "sqlserver")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "tsv")]
        let driver_count = driver_count + 1;

        assert_eq!(driver_manager.drivers.len(), driver_count);
    }

    #[tokio::test]
    async fn test_driver_manager_connect_with_colon() -> anyhow::Result<()> {
        let identifier = "test";
        let mut mock_driver = MockDriver::new();
        mock_driver.expect_identifier().returning(|| identifier);
        mock_driver
            .expect_connect()
            .returning(|_, _| Ok(Box::new(MockConnection::new())));

        let mut driver_manager = DriverManager::new();
        driver_manager.add(Box::new(mock_driver));

        let _ = driver_manager.connect("test:").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_driver_manager_connect_without_colon() {
        let driver_manager = DriverManager::new();
        assert!(driver_manager.connect("test").await.is_err());
    }

    #[tokio::test]
    async fn test_driver_manager_connect_error() {
        let driver_manager = DriverManager::default();
        let result = driver_manager.connect("foo").await;
        assert!(result.is_err());
    }
}
