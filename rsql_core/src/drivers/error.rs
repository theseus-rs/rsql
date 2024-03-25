pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error when a driver for an identifier is not found
    #[error("driver not found for identifier [{identifier}]")]
    DriverNotFound { identifier: String },
    /// Error parsing a URL
    #[error(transparent)]
    InvalidUrl(#[from] url::ParseError),
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

/// Converts a [`rusqlite::Error`] into an [`ParseError`](Error::IoError)
#[cfg(feature = "rusqlite")]
impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_archive_error() {
        let error = postgresql_archive::Error::Unexpected("test".to_string());
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "test");
    }

    #[test]
    fn test_embedded_error() {
        let archive_error = postgresql_archive::Error::Unexpected("test".to_string());
        let error = postgresql_embedded::Error::ArchiveError(archive_error);
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "test");
    }

    #[test]
    fn test_rusqlite_error() {
        let error = rusqlite::Error::QueryReturnedNoRows;
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "Query returned no rows");
    }

    #[test]
    fn test_sqlx_error() {
        let error = sqlx::Error::RowNotFound;
        let io_error = Error::from(error);

        assert!(io_error.to_string().contains("no rows returned"));
    }
}
