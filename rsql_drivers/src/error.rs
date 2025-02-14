pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error when a driver fails
    #[error(transparent)]
    DriverError(#[from] rsql_driver::Error),
    /// Error when a driver for an identifier is not found
    #[error("driver not found for: {0}")]
    DriverNotFound(String),
}
