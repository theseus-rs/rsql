pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Command error
    #[error(transparent)]
    CommandError(#[from] crate::commands::Error),
    /// Driver error
    #[error(transparent)]
    DriverError(#[from] rsql_drivers::Error),
    /// Format error
    #[error(transparent)]
    FormatError(#[from] rsql_formatters::Error),
    /// Error when an invalid command is given
    #[error("Invalid command {command_name}")]
    InvalidCommand { command_name: String },
    /// IO error
    #[error(transparent)]
    IoError(anyhow::Error),
}

/// Converts a [indicatif::style::TemplateError] into an [IoError](Error::IoError)
impl From<indicatif::style::TemplateError> for Error {
    fn from(error: indicatif::style::TemplateError) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [regex::Error] into an [IoError](Error::IoError)
impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [std::io::Error] into an [IoError](Error::IoError)
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_template_error() {
        let result = indicatif::ProgressStyle::with_template("{:^3");
        let error = result.err().expect("Error");
        let template_error = Error::from(error);
        assert!(template_error.to_string().contains(":"));
    }

    #[test]
    fn test_regex_error() {
        let error = regex::Error::Syntax("test".to_string());
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "test");
    }

    #[test]
    fn test_std_io_error() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "test");
    }
}
