pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error
    #[error(transparent)]
    IoError(anyhow::Error),
    /// Error an unknown format is specified
    #[error("unknown format [{format}]")]
    UnknownFormat { format: String },
}

#[cfg(any(
    feature = "ascii",
    feature = "markdown",
    feature = "plain",
    feature = "psql",
    feature = "unicode"
))]
/// Converts a [`csv::Error`] into an [`IoError`](Error::IoError)
impl From<csv::Error> for Error {
    fn from(error: csv::Error) -> Self {
        Error::IoError(error.into())
    }
}

#[cfg(any(feature = "html", feature = "xml"))]
/// Converts a [`quick_xml::Error`] into an [`IoError`](Error::IoError)
impl From<quick_xml::Error> for Error {
    fn from(error: quick_xml::Error) -> Self {
        Error::IoError(error.into())
    }
}

#[cfg(any(feature = "json", feature = "jsonl"))]
/// Converts a [`serde_json::Error`] into an [`IoError`](Error::IoError)
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::IoError(error.into())
    }
}

#[cfg(feature = "yaml")]
/// Converts a [`serde_yaml::Error`] into an [`IoError`](Error::IoError)
impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`std::io::Error`] into an [`IoError`](Error::IoError)
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error.into())
    }
}

/// Convert [utf8 errors](std::string::FromUtf8Error) to [embedded errors](Error::IoError)
impl From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Error::IoError(error.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[cfg(any(
        feature = "ascii",
        feature = "markdown",
        feature = "plain",
        feature = "psql",
        feature = "unicode"
    ))]
    #[test]
    fn test_csv_error() {
        let std_io_error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let csv_error = csv::Error::from(std_io_error);
        let io_error = Error::from(csv_error);
        assert_eq!(io_error.to_string(), "test");
    }

    #[cfg(any(feature = "html", feature = "xml"))]
    #[test]
    fn test_quick_xml_error() {
        let error = quick_xml::Error::UnexpectedToken("test".to_string());
        let io_error = Error::from(error);
        assert_eq!(io_error.to_string(), "Unexpected token 'test'");
    }

    #[cfg(any(feature = "json", feature = "jsonl"))]
    #[test]
    fn test_serde_json_error() {
        let serde_json_error = serde_json::from_str::<String>(">").unwrap_err();
        let io_error = Error::from(serde_json_error);
        assert_eq!(io_error.to_string(), "expected value at line 1 column 1");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_serde_yaml_error() {
        let serde_yaml_error = serde_yaml::from_str::<String>(">\n@").unwrap_err();
        let io_error = Error::from(serde_yaml_error);
        assert_eq!(io_error.to_string(), "found character that cannot start any token at line 2 column 1, while scanning for the next token");
    }

    #[test]
    fn test_std_io_error() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let io_error = Error::from(error);
        assert_eq!(io_error.to_string(), "test");
    }

    #[test]
    fn test_from_utf8_error() {
        let invalid_utf8: Vec<u8> = vec![0, 159, 146, 150];
        let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
        let error = Error::from(utf8_error);
        assert_eq!(
            error.to_string(),
            "invalid utf-8 sequence of 1 bytes from index 1"
        );
    }
}
