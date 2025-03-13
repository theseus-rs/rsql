use crate::Error::{DriverNotFound, InvalidUrl, IoError};
use crate::error::Result;
use crate::{CachedMetadataConnection, Connection, Driver};
use file_type::FileType;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::{Arc, LazyLock, RwLock};
use tracing::instrument;
use url::Url;

type DriverMap = BTreeMap<&'static str, Arc<dyn Driver>>;

static DRIVERS: LazyLock<Arc<RwLock<DriverMap>>> =
    LazyLock::new(|| Arc::new(RwLock::new(BTreeMap::new())));

/// Manages available drivers
#[derive(Debug)]
pub struct DriverManager {}

impl DriverManager {
    /// Add a new driver to the list of available drivers
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn add(driver: Arc<dyn Driver>) -> Result<()> {
        let identifier = driver.identifier();
        let mut drivers = DRIVERS
            .write()
            .map_err(|error| IoError(error.to_string()))?;
        let _ = drivers.insert(identifier, driver);
        Ok(())
    }

    /// Get a drivers by name
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn get<S: AsRef<str>>(identifier: S) -> Result<Option<Arc<dyn Driver>>> {
        let identifier = identifier.as_ref();
        let drivers = DRIVERS.read().map_err(|error| IoError(error.to_string()))?;
        let Some(driver) = drivers.get(identifier) else {
            return Ok(None);
        };
        Ok(Some(driver.clone()))
    }

    /// Get a drivers by name
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn get_by_file_type(file_type: &FileType) -> Result<Option<Arc<dyn Driver>>> {
        let drivers = DRIVERS.read().map_err(|error| IoError(error.to_string()))?;
        let Some(driver) = drivers
            .iter()
            .find(|(_, driver)| driver.supports_file_type(file_type))
            .map(|(_, driver)| driver.clone())
        else {
            return Ok(None);
        };
        Ok(Some(driver))
    }

    /// Get all drivers
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn drivers() -> Result<Vec<Arc<dyn Driver>>> {
        let drivers = DRIVERS.read().map_err(|error| IoError(error.to_string()))?;
        Ok(drivers.values().cloned().collect())
    }

    /// Connect to a database
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    #[instrument(name = "connect", level = "info", skip(url))]
    pub async fn connect<S: AsRef<str>>(url: S) -> Result<Box<dyn Connection>> {
        let url = url.as_ref();
        let parsed_url = Url::parse(url).map_err(|error| InvalidUrl(error.to_string()))?;
        let scheme = parsed_url.scheme();

        match Self::get(scheme)? {
            Some(driver) => {
                let connection = driver.connect(url).await?;
                let connection = Box::new(CachedMetadataConnection::new(connection));
                Ok(connection)
            }
            None => Err(DriverNotFound(scheme.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MockConnection, MockDriver};

    const IDENTIFIER: &str = "test";

    fn add_mock_driver() -> Result<()> {
        let mut mock_driver = MockDriver::new();
        mock_driver.expect_identifier().returning(|| IDENTIFIER);
        mock_driver.expect_supports_file_type().returning(|_| false);
        mock_driver
            .expect_connect()
            .returning(|_| Ok(Box::new(MockConnection::new())));
        DriverManager::add(Arc::new(mock_driver))?;
        Ok(())
    }

    #[test]
    fn test_add() -> Result<()> {
        let drivers = DriverManager::drivers()?;
        assert!(drivers.is_empty());

        add_mock_driver()?;

        let drivers = DriverManager::drivers()?;
        assert_eq!(drivers.len(), 1);
        let result = DriverManager::get(IDENTIFIER)?;
        assert!(result.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_connect_with_colon() -> Result<()> {
        add_mock_driver()?;
        let _ = DriverManager::connect(format!("{IDENTIFIER}:").as_str()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connect_without_colon() -> Result<()> {
        add_mock_driver()?;
        let result = DriverManager::connect(IDENTIFIER).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_connect_error() {
        let result = DriverManager::connect("foo").await;
        assert!(result.is_err());
    }
}
