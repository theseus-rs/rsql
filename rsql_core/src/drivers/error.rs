pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error when a driver for an identifier is not found
    #[error("driver not found for identifier [{identifier}]")]
    DriverNotFound { identifier: String },
    /// IO error
    #[error(transparent)]
    IoError(anyhow::Error),
    /// Error when a column type is not supported
    #[error("column type [{column_type}] is not supported for column [{column_name}]")]
    UnsupportedColumnType {
        column_name: String,
        column_type: String,
    },
}

/// Converts a [`postgresql_archive::Error`] into an [`IoError`](Error::IoError)
#[cfg(feature = "postgresql")]
impl From<postgresql_archive::Error> for Error {
    fn from(error: postgresql_archive::Error) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`postgresql_embedded::Error`] into an [`IoError`](Error::IoError)
#[cfg(feature = "postgresql")]
impl From<postgresql_embedded::Error> for Error {
    fn from(error: postgresql_embedded::Error) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`sqlx::Error`] into an [`ParseError`](Error::IoError)
#[cfg(any(feature = "postgresql", feature = "sqlite"))]
impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::IoError(error.into())
    }
}
