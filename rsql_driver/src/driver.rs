use crate::Connection;
use crate::error::Result;
use async_trait::async_trait;
use file_type::FileType;
use mockall::automock;
use mockall::predicate::str;
use std::fmt::Debug;

/// The `Driver` trait defines the interface for connecting to different data sources and executing
/// SQL queries.
#[automock]
#[async_trait]
pub trait Driver: Debug + Send + Sync {
    /// Returns the identifier of the driver.  The idenfitier is used as the scheme in the URL to
    /// identify the driver.
    fn identifier(&self) -> &'static str;

    /// Connects to the data source using the specified URL.
    async fn connect(&self, url: &str) -> Result<Box<dyn Connection>>;

    /// Returns whether the driver supports the specified file type.
    fn supports_file_type(&self, file_type: &FileType) -> bool;
}
