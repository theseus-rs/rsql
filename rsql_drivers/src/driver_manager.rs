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
        #[cfg(feature = "driver-arrow")]
        Self::add(Arc::new(rsql_driver_arrow::Driver))?;
        #[cfg(feature = "driver-avro")]
        Self::add(Arc::new(rsql_driver_avro::Driver))?;
        #[cfg(feature = "driver-brotli")]
        Self::add(Arc::new(rsql_driver_brotli::Driver))?;
        #[cfg(feature = "driver-bzip2")]
        Self::add(Arc::new(rsql_driver_bzip2::Driver))?;
        #[cfg(feature = "driver-cockroachdb")]
        Self::add(Arc::new(rsql_driver_cockroachdb::Driver))?;
        #[cfg(feature = "driver-csv")]
        Self::add(Arc::new(rsql_driver_csv::Driver))?;
        #[cfg(feature = "driver-delimited")]
        Self::add(Arc::new(rsql_driver_delimited::Driver))?;
        #[cfg(feature = "driver-duckdb")]
        Self::add(Arc::new(rsql_driver_duckdb::Driver))?;
        #[cfg(feature = "driver-dynamodb")]
        Self::add(Arc::new(rsql_driver_dynamodb::Driver))?;
        #[cfg(feature = "driver-excel")]
        Self::add(Arc::new(rsql_driver_excel::Driver))?;
        #[cfg(feature = "driver-file")]
        Self::add(Arc::new(rsql_driver_file::Driver))?;
        #[cfg(feature = "driver-fwf")]
        Self::add(Arc::new(rsql_driver_fwf::Driver))?;
        #[cfg(feature = "driver-gzip")]
        Self::add(Arc::new(rsql_driver_gzip::Driver))?;
        #[cfg(feature = "driver-http")]
        Self::add(Arc::new(rsql_driver_http::Driver))?;
        #[cfg(feature = "driver-https")]
        Self::add(Arc::new(rsql_driver_https::Driver))?;
        #[cfg(feature = "driver-json")]
        Self::add(Arc::new(rsql_driver_json::Driver))?;
        #[cfg(feature = "driver-jsonl")]
        Self::add(Arc::new(rsql_driver_jsonl::Driver))?;
        #[cfg(feature = "driver-libsql")]
        Self::add(Arc::new(rsql_driver_libsql::Driver))?;
        #[cfg(feature = "driver-lz4")]
        Self::add(Arc::new(rsql_driver_lz4::Driver))?;
        #[cfg(feature = "driver-mariadb")]
        Self::add(Arc::new(rsql_driver_mariadb::Driver))?;
        #[cfg(feature = "driver-mysql")]
        Self::add(Arc::new(rsql_driver_mysql::Driver))?;
        #[cfg(feature = "driver-ods")]
        Self::add(Arc::new(rsql_driver_ods::Driver))?;
        #[cfg(feature = "driver-orc")]
        Self::add(Arc::new(rsql_driver_orc::Driver))?;
        #[cfg(feature = "driver-parquet")]
        Self::add(Arc::new(rsql_driver_parquet::Driver))?;
        #[cfg(feature = "driver-postgres")]
        Self::add(Arc::new(rsql_driver_postgres::Driver))?;
        #[cfg(feature = "driver-postgresql")]
        Self::add(Arc::new(rsql_driver_postgresql::Driver))?;
        #[cfg(feature = "driver-redshift")]
        Self::add(Arc::new(rsql_driver_redshift::Driver))?;
        #[cfg(feature = "driver-rusqlite")]
        Self::add(Arc::new(rsql_driver_rusqlite::Driver))?;
        #[cfg(feature = "driver-s3")]
        Self::add(Arc::new(rsql_driver_s3::Driver))?;
        #[cfg(feature = "driver-snowflake")]
        Self::add(Arc::new(rsql_driver_snowflake::Driver))?;
        #[cfg(feature = "driver-sqlite")]
        Self::add(Arc::new(rsql_driver_sqlite::Driver))?;
        #[cfg(feature = "driver-sqlserver")]
        Self::add(Arc::new(rsql_driver_sqlserver::Driver))?;
        #[cfg(feature = "driver-tsv")]
        Self::add(Arc::new(rsql_driver_tsv::Driver))?;
        #[cfg(feature = "driver-xml")]
        Self::add(Arc::new(rsql_driver_xml::Driver))?;
        #[cfg(feature = "driver-xz")]
        Self::add(Arc::new(rsql_driver_xz::Driver))?;
        #[cfg(feature = "driver-yaml")]
        Self::add(Arc::new(rsql_driver_yaml::Driver))?;
        #[cfg(feature = "driver-zstd")]
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
        #[cfg(feature = "driver-arrow")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-avro")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-brotli")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-bzip2")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-cockroachdb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-csv")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-delimited")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-duckdb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-dynamodb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-excel")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-file")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-fwf")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-gzip")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-http")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-https")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-json")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-jsonl")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-libsql")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-lz4")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-mariadb")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-mysql")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-ods")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-orc")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-parquet")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-postgres")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-postgresql")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-redshift")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-rusqlite")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-s3")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-snowflake")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-sqlite")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-sqlserver")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-tsv")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-xml")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-xz")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-yaml")]
        let driver_count = driver_count + 1;
        #[cfg(feature = "driver-zstd")]
        let driver_count = driver_count + 1;

        let drivers = DriverManager::drivers()?;
        // The number of drivers should be at least the number of features enabled.  This value may
        // be higher if additional drivers are added during testing.
        assert!(drivers.len() >= driver_count);
        Ok(())
    }
}
