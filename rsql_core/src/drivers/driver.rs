use crate::configuration::Configuration;
use crate::drivers::error::Result;
use crate::drivers::Connection;
use crate::drivers::Error::DriverNotFound;
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::fmt::Debug;
use tracing::instrument;
use url::Url;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Driver: Debug + Send + Sync {
    fn identifier(&self) -> &'static str;
    async fn connect(
        &self,
        configuration: &Configuration,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn Connection>>;
}

/// Manages available drivers
#[derive(Debug)]
pub struct DriverManager {
    drivers: BTreeMap<&'static str, Box<dyn Driver>>,
}

impl DriverManager {
    /// Create a new instance of the `DriverManager`
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
    pub fn get(&self, identifier: &str) -> Option<&dyn Driver> {
        self.drivers.get(identifier).map(|driver| driver.as_ref())
    }

    /// Get an iterator over the available drivers
    pub fn iter(&self) -> impl Iterator<Item = &dyn Driver> {
        self.drivers.values().map(|driver| driver.as_ref())
    }

    /// Connect to a database
    #[instrument(name = "connect", level = "info", skip(configuration, url))]
    pub async fn connect(
        &self,
        configuration: &Configuration,
        url: &str,
    ) -> Result<Box<dyn Connection>> {
        let parsed_url = Url::parse(url)?;
        let scheme = parsed_url.scheme();
        let password = parsed_url.password().map(|password| password.to_string());
        let url = url.to_string();

        match &self.get(scheme) {
            Some(driver) => driver.connect(configuration, url, password).await,
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

        #[cfg(any(feature = "postgresql", feature = "sqlite"))]
        sqlx::any::install_default_drivers();

        #[cfg(feature = "postgresql")]
        drivers.add(Box::new(crate::drivers::postgresql::Driver));
        #[cfg(feature = "rusqlite")]
        drivers.add(Box::new(crate::drivers::rusqlite::Driver));
        #[cfg(feature = "sqlite")]
        drivers.add(Box::new(crate::drivers::sqlite::Driver));

        drivers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::MockConnection;

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

        #[cfg(feature = "postgresql")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "rusqlite")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "sqlite")]
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
            .returning(|_, _, _| Ok(Box::new(MockConnection::new())));

        let mut driver_manager = DriverManager::new();
        driver_manager.add(Box::new(mock_driver));

        let configuration = Configuration::default();
        let _ = driver_manager.connect(&configuration, "test:").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_driver_manager_connect_without_colon() {
        let driver_manager = DriverManager::new();
        let configuration = Configuration::default();
        assert!(driver_manager
            .connect(&configuration, "test")
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_driver_manager_connect_error() {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let result = driver_manager.connect(&configuration, "foo").await;
        assert!(result.is_err());
    }
}
