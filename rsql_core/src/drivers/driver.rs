use crate::configuration::Configuration;
use crate::drivers::Connection;
use anyhow::bail;
use async_trait::async_trait;
use sqlx::any::install_default_drivers;
use std::collections::BTreeMap;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Driver: Send {
    fn identifier(&self) -> &'static str;
    async fn connect(
        &self,
        configuration: &Configuration,
        url: &str,
    ) -> anyhow::Result<Box<dyn Connection>>;
}

/// Manages available drivers
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
    fn add(&mut self, driver: Box<dyn Driver>) {
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
    pub async fn connect(
        &self,
        configuration: &Configuration,
        url: &str,
    ) -> anyhow::Result<Box<dyn Connection>> {
        let identifier = match url.split_once(':') {
            Some((before, _)) => before,
            None => "",
        };

        match &self.get(identifier) {
            Some(driver) => driver.connect(configuration, url).await,
            None => bail!("Invalid database url: {url}"),
        }
    }
}

/// Default implementation for the `DriverManager`
impl Default for DriverManager {
    fn default() -> Self {
        let mut drivers = DriverManager::new();

        #[cfg(any(feature = "postgresql", feature = "sqlite"))]
        install_default_drivers();

        #[cfg(feature = "postgresql")]
        drivers.add(Box::new(crate::drivers::postgresql::Driver));
        #[cfg(feature = "sqlite")]
        drivers.add(Box::new(crate::drivers::sqlite::Driver));

        drivers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

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
        let drivers = DriverManager::default();
        let driver_count = 0;

        #[cfg(feature = "postgresql")]
        let driver_count = driver_count + 1;

        #[cfg(feature = "sqlite")]
        let driver_count = driver_count + 1;

        assert_eq!(drivers.drivers.len(), driver_count);
    }

    #[tokio::test]
    async fn test_driver_manager_connect() -> Result<()> {
        let configuration = Configuration::default();
        let drivers = DriverManager::default();
        let mut connection = drivers.connect(&configuration, "sqlite::memory:").await?;
        connection.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_driver_manager_connect_error() {
        let configuration = Configuration::default();
        let drivers = DriverManager::default();
        let result = drivers.connect(&configuration, "foo").await;
        assert!(result.is_err());
    }
}
