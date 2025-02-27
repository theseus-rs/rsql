pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Data type conversion error
    #[error("{0}")]
    ConversionError(String),
    /// Error parsing a URL
    #[error("{0}")]
    InvalidUrl(String),
    /// IO error
    #[error("{0}")]
    IoError(String),
    /// Error when parsing an integer
    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),
    /// Error when a column type is not supported
    #[error("column type [{column_type}] is not supported for column [{column_name}]")]
    UnsupportedColumnType {
        column_name: String,
        column_type: String,
    },
}

/// Converts a [`std::io::Error`] into an [`IoError`](Error::IoError)
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error.to_string())
    }
}

/// Convert [`utf8 errors`](std::string::FromUtf8Error) to [`IoError`](Error::IoError)
impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Error::IoError(error.to_string())
    }
}

/// Convert [`url::ParseError`] to [`IoError`](Error::IoError)
impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::InvalidUrl(error.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_std_io_error() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "test");
    }

    #[test]
    fn test_from_utf8_error() {
        let invalid_utf8: Vec<u8> = vec![0, 159, 146, 150];
        let utf8_error = String::from_utf8(invalid_utf8).expect_err("expected FromUtf8Error");
        let error = Error::from(utf8_error);
        assert_eq!(
            error.to_string(),
            "invalid utf-8 sequence of 1 bytes from index 1"
        );
    }

    #[test]
    fn test_from_url_parse_error() {
        let error = url::ParseError::EmptyHost;
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "empty host");
    }
}
