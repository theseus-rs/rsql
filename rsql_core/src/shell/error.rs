pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Command error
    #[error(transparent)]
    CommandError(#[from] crate::commands::Error),
    /// Driver error
    #[error(transparent)]
    DriverError(#[from] crate::drivers::Error),
    /// Format error
    #[error(transparent)]
    FormatError(#[from] crate::formatters::Error),
    /// IO error
    #[error(transparent)]
    IoError(anyhow::Error),
    /// Unknown error
    #[error(transparent)]
    UnknownError(anyhow::Error),
}

/// Converts a [`clap_stdin::StdinError`] into an [`IoError`](Error::IoError)
impl From<clap_stdin::StdinError> for Error {
    fn from(error: clap_stdin::StdinError) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`indicatif::style::TemplateError`] into an [`IoError`](Error::IoError)
impl From<indicatif::style::TemplateError> for Error {
    fn from(error: indicatif::style::TemplateError) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`regex::Error`] into an [`IoError`](Error::IoError)
impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`rustyline::error::ReadlineError`] into an [`IoError`](Error::IoError)
impl From<rustyline::error::ReadlineError> for Error {
    fn from(error: rustyline::error::ReadlineError) -> Self {
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
    use indicatif::ProgressStyle;
    use test_log::test;

    #[test]
    fn test_stdin_error() {
        let error = clap_stdin::StdinError::FromStr("test".to_string());
        let io_error = Error::from(error);

        assert!(io_error.to_string().contains("test"));
    }

    #[test]
    fn test_template_error() {
        match ProgressStyle::with_template("{:^3") {
            Ok(_) => panic!("expected error"),
            Err(error) => {
                let template_error = Error::from(error);
                assert!(template_error.to_string().contains(":"));
            }
        }
    }

    #[test]
    fn test_regex_error() {
        let error = regex::Error::Syntax("test".to_string());
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "test");
    }

    #[test]
    fn test_rusty_line_error() {
        let std_io_error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let error = rustyline::error::ReadlineError::Io(std_io_error);
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
