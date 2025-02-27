use crate::Error::DriverNotFound;
use crate::error::Result;
use file_type::FileType;
use rsql_driver::Error::InvalidUrl;
use rsql_driver::{CachedMetadataConnection, Connection, Driver};
use std::collections::BTreeMap;
use std::fmt::Debug;
use tracing::instrument;
use url::Url;

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

    /// Get a drivers by name
    #[must_use]
    pub fn get_by_file_type(&self, file_type: &FileType) -> Option<&dyn Driver> {
        self.drivers
            .iter()
            .find(|(_, driver)| driver.supports_file_type(file_type))
            .map(|(_, driver)| driver.as_ref())
    }

    /// Get an iterator over the available drivers
    pub fn iter(&self) -> impl Iterator<Item = &dyn Driver> {
        self.drivers.values().map(AsRef::as_ref)
    }

    /// Connect to a database
    #[instrument(name = "connect", level = "info", skip(url))]
    pub async fn connect(&self, url: &str) -> Result<Box<dyn Connection>> {
        let parsed_url = Url::parse(url).map_err(|error| InvalidUrl(error.to_string()))?;
        let scheme = parsed_url.scheme();
        let url = url.to_string();

        match &self.get(scheme) {
            Some(driver) => {
                let connection = driver.connect(&url).await?;
                let connection = Box::new(CachedMetadataConnection::new(connection));
                Ok(connection)
            }
            None => Err(DriverNotFound(scheme.to_string())),
        }
    }
}

/// Default implementation for the `DriverManager`
impl Default for DriverManager {
    fn default() -> Self {
        let mut drivers = DriverManager::new();

        #[cfg(feature = "arrow")]
        drivers.add(Box::new(rsql_driver_arrow::Driver));
        #[cfg(feature = "avro")]
        drivers.add(Box::new(rsql_driver_avro::Driver));
        #[cfg(feature = "cockroachdb")]
        drivers.add(Box::new(rsql_driver_cockroachdb::Driver));
        #[cfg(feature = "csv")]
        drivers.add(Box::new(rsql_driver_csv::Driver));
        #[cfg(feature = "delimited")]
        drivers.add(Box::new(rsql_driver_delimited::Driver));
        #[cfg(feature = "duckdb")]
        drivers.add(Box::new(rsql_driver_duckdb::Driver));
        #[cfg(feature = "excel")]
        drivers.add(Box::new(rsql_driver_excel::Driver));
        #[cfg(feature = "file")]
        drivers.add(Box::new(crate::file::Driver));
        #[cfg(feature = "http")]
        drivers.add(Box::new(crate::http::Driver));
        #[cfg(feature = "https")]
        drivers.add(Box::new(crate::https::Driver));
        #[cfg(feature = "json")]
        drivers.add(Box::new(rsql_driver_json::Driver));
        #[cfg(feature = "jsonl")]
        drivers.add(Box::new(rsql_driver_jsonl::Driver));
        #[cfg(feature = "libsql")]
        drivers.add(Box::new(rsql_driver_libsql::Driver));
        #[cfg(feature = "mariadb")]
        drivers.add(Box::new(rsql_driver_mariadb::Driver));
        #[cfg(feature = "mysql")]
        drivers.add(Box::new(rsql_driver_mysql::Driver));
        #[cfg(feature = "ods")]
        drivers.add(Box::new(rsql_driver_ods::Driver));
        #[cfg(feature = "parquet")]
        drivers.add(Box::new(rsql_driver_parquet::Driver));
        #[cfg(feature = "postgres")]
        drivers.add(Box::new(rsql_driver_postgres::Driver));
        #[cfg(feature = "postgresql")]
        drivers.add(Box::new(rsql_driver_postgresql::Driver));
        #[cfg(feature = "redshift")]
        drivers.add(Box::new(rsql_driver_redshift::Driver));
        #[cfg(feature = "rusqlite")]
        drivers.add(Box::new(rsql_driver_rusqlite::Driver));
        #[cfg(feature = "snowflake")]
        drivers.add(Box::new(rsql_driver_snowflake::Driver));
        #[cfg(feature = "sqlite")]
        drivers.add(Box::new(rsql_driver_sqlite::Driver));
        #[cfg(feature = "sqlserver")]
        drivers.add(Box::new(rsql_driver_sqlserver::Driver));
        #[cfg(feature = "tsv")]
        drivers.add(Box::new(rsql_driver_tsv::Driver));
        #[cfg(feature = "xml")]
        drivers.add(Box::new(rsql_driver_xml::Driver));
        #[cfg(feature = "yaml")]
        drivers.add(Box::new(rsql_driver_yaml::Driver));

        drivers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsql_driver::MockConnection;
    use rsql_driver::MockDriver;

    #[test]
    fn test_driver_manager() {
        let identifier = "test";
        let mut mock_driver = MockDriver::new();
        mock_driver.expect_identifier().returning(|| identifier);
        mock_driver.expect_supports_file_type().returning(|_| false);

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
        #[cfg(feature = "avro")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "cockroachdb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "csv")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "delimited")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "duckdb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "excel")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "file")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "http")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "https")]
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
        #[cfg(feature = "ods")]
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
        #[cfg(feature = "xml")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "yaml")]
        let driver_count = driver_count + 1;

        assert_eq!(driver_manager.drivers.len(), driver_count);
    }

    #[tokio::test]
    async fn test_driver_manager_connect_with_colon() -> Result<()> {
        let identifier = "test";
        let mut mock_driver = MockDriver::new();
        mock_driver.expect_identifier().returning(|| identifier);
        mock_driver.expect_supports_file_type().returning(|_| false);
        mock_driver
            .expect_connect()
            .returning(|_| Ok(Box::new(MockConnection::new())));

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
