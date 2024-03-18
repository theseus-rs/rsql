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

/// Converts a [`csv::Error`] into an [`IoError`](Error::IoError)
impl From<csv::Error> for Error {
    fn from(error: csv::Error) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`quick_xml::Error`] into an [`IoError`](Error::IoError)
impl From<quick_xml::Error> for Error {
    fn from(error: quick_xml::Error) -> Self {
        Error::IoError(error.into())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_csv_error() {
        let std_io_error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let csv_error = csv::Error::from(std_io_error);
        let io_error = Error::from(csv_error);

        assert_eq!(io_error.to_string(), "test");
    }

    #[test]
    fn test_quick_xml_error() {
        let error = quick_xml::Error::UnexpectedToken("test".to_string());
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "Unexpected token 'test'");
    }

    #[test]
    fn test_serde_yaml_error() {
        match serde_yaml::from_str::<String>(">\n@") {
            Ok(_) => panic!("expected error"),
            Err(error) => {
                let io_error = Error::from(error);
                assert_eq!(io_error.to_string(), "found character that cannot start any token at line 2 column 1, while scanning for the next token");
            }
        }
    }

    #[test]
    fn test_std_io_error() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "test");
    }
}
