use file_type::FileType;
use rsql_driver::{Connection, Driver, Result};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::instrument;

/// Manages available drivers
#[derive(Debug)]
pub struct DriverManager {}

impl DriverManager {
    /// Add a new driver to the list of available drivers
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn add(driver: Arc<dyn Driver>) -> Result<()> {
        rsql_driver::DriverManager::add(driver)
    }

    /// Get a drivers by name
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn get<S: AsRef<str>>(identifier: S) -> Result<Option<Arc<dyn Driver>>> {
        rsql_driver::DriverManager::get(identifier)
    }

    /// Get a drivers by name
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn get_by_file_type(file_type: &FileType) -> Result<Option<Arc<dyn Driver>>> {
        rsql_driver::DriverManager::get_by_file_type(file_type)
    }

    /// Get all drivers
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn drivers() -> Result<Vec<Arc<dyn Driver>>> {
        rsql_driver::DriverManager::drivers()
    }

    /// Connect to a database
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    #[instrument(name = "connect", level = "info", skip(url))]
    pub async fn connect<S: AsRef<str>>(url: S) -> Result<Box<dyn Connection>> {
        rsql_driver::DriverManager::connect(url).await
    }

    /// Initialize known drivers based on enabled features
    ///
    /// # Errors
    /// * If a lock for drivers cannot be acquired
    pub fn initialize() -> Result<()> {
        #[cfg(feature = "arrow")]
        Self::add(Arc::new(rsql_driver_arrow::Driver))?;
        #[cfg(feature = "avro")]
        Self::add(Arc::new(rsql_driver_avro::Driver))?;
        #[cfg(feature = "brotli")]
        Self::add(Arc::new(rsql_driver_brotli::Driver))?;
        #[cfg(feature = "bzip2")]
        Self::add(Arc::new(rsql_driver_bzip2::Driver))?;
        #[cfg(feature = "cockroachdb")]
        Self::add(Arc::new(rsql_driver_cockroachdb::Driver))?;
        #[cfg(feature = "csv")]
        Self::add(Arc::new(rsql_driver_csv::Driver))?;
        #[cfg(feature = "delimited")]
        Self::add(Arc::new(rsql_driver_delimited::Driver))?;
        #[cfg(feature = "duckdb")]
        Self::add(Arc::new(rsql_driver_duckdb::Driver))?;
        #[cfg(feature = "dynamodb")]
        Self::add(Arc::new(rsql_driver_dynamodb::Driver))?;
        #[cfg(feature = "excel")]
        Self::add(Arc::new(rsql_driver_excel::Driver))?;
        #[cfg(feature = "file")]
        Self::add(Arc::new(rsql_driver_file::Driver))?;
        #[cfg(feature = "fwf")]
        Self::add(Arc::new(rsql_driver_fwf::Driver))?;
        #[cfg(feature = "gzip")]
        Self::add(Arc::new(rsql_driver_gzip::Driver))?;
        #[cfg(feature = "http")]
        Self::add(Arc::new(rsql_driver_http::Driver))?;
        #[cfg(feature = "https")]
        Self::add(Arc::new(rsql_driver_https::Driver))?;
        #[cfg(feature = "json")]
        Self::add(Arc::new(rsql_driver_json::Driver))?;
        #[cfg(feature = "jsonl")]
        Self::add(Arc::new(rsql_driver_jsonl::Driver))?;
        #[cfg(feature = "libsql")]
        Self::add(Arc::new(rsql_driver_libsql::Driver))?;
        #[cfg(feature = "lz4")]
        Self::add(Arc::new(rsql_driver_lz4::Driver))?;
        #[cfg(feature = "mariadb")]
        Self::add(Arc::new(rsql_driver_mariadb::Driver))?;
        #[cfg(feature = "mysql")]
        Self::add(Arc::new(rsql_driver_mysql::Driver))?;
        #[cfg(feature = "ods")]
        Self::add(Arc::new(rsql_driver_ods::Driver))?;
        #[cfg(feature = "orc")]
        Self::add(Arc::new(rsql_driver_orc::Driver))?;
        #[cfg(feature = "parquet")]
        Self::add(Arc::new(rsql_driver_parquet::Driver))?;
        #[cfg(feature = "postgres")]
        Self::add(Arc::new(rsql_driver_postgres::Driver))?;
        #[cfg(feature = "postgresql")]
        Self::add(Arc::new(rsql_driver_postgresql::Driver))?;
        #[cfg(feature = "redshift")]
        Self::add(Arc::new(rsql_driver_redshift::Driver))?;
        #[cfg(feature = "rusqlite")]
        Self::add(Arc::new(rsql_driver_rusqlite::Driver))?;
        #[cfg(feature = "s3")]
        Self::add(Arc::new(rsql_driver_s3::Driver))?;
        #[cfg(feature = "snowflake")]
        Self::add(Arc::new(rsql_driver_snowflake::Driver))?;
        #[cfg(feature = "sqlite")]
        Self::add(Arc::new(rsql_driver_sqlite::Driver))?;
        #[cfg(feature = "sqlserver")]
        Self::add(Arc::new(rsql_driver_sqlserver::Driver))?;
        #[cfg(feature = "tsv")]
        Self::add(Arc::new(rsql_driver_tsv::Driver))?;
        #[cfg(feature = "xml")]
        Self::add(Arc::new(rsql_driver_xml::Driver))?;
        #[cfg(feature = "xz")]
        Self::add(Arc::new(rsql_driver_xz::Driver))?;
        #[cfg(feature = "yaml")]
        Self::add(Arc::new(rsql_driver_yaml::Driver))?;
        #[cfg(feature = "zstd")]
        Self::add(Arc::new(rsql_driver_zstd::Driver))?;
        Ok(())
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
        add_mock_driver()?;
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

    #[test]
    fn test_initialize() -> Result<()> {
        DriverManager::initialize()?;
        let driver_count = 0;
        #[cfg(feature = "arrow")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "avro")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "brotli")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "bzip2")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "cockroachdb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "csv")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "delimited")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "duckdb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "dynamodb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "excel")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "file")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "fwf")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "gzip")]
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
        #[cfg(feature = "lz4")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "mariadb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "mysql")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "ods")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "orc")]
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
        #[cfg(feature = "s3")]
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
        #[cfg(feature = "xz")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "yaml")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "zstd")]
        let driver_count = driver_count + 1;

        let drivers = DriverManager::drivers()?;
        // The number of drivers should be at least the number of features enabled.  This value may
        // be higher if additional drivers are added during testing.
        assert!(drivers.len() >= driver_count);
        Ok(())
    }
}
