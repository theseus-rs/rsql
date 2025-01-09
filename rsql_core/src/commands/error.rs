use crate::commands::LoopCondition;

pub type Result<T = LoopCondition, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Driver error
    #[error(transparent)]
    DriverError(#[from] rsql_drivers::Error),
    /// Formatter error
    #[error(transparent)]
    FormatterError(#[from] rsql_formatters::Error),
    /// Error when an invalid option is provided for a command
    #[error("Invalid {command_name} option: {option}")]
    InvalidOption {
        command_name: String,
        option: String,
    },
    /// IO error
    #[error(transparent)]
    IoError(anyhow::Error),
    /// Error when a command is missing required arguments
    #[error("{command_name} is missing a required argument: {arguments}")]
    MissingArguments {
        command_name: String,
        arguments: String,
    },
}

/// Converts a [`clearscreen::Error`] into an [`IoError`](Error::IoError)
impl From<clearscreen::Error> for Error {
    fn from(error: clearscreen::Error) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`std::num::ParseIntError`] into an [`IoError`](Error::IoError)
impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
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
    use std::str::FromStr;
    use test_log::test;

    #[test]
    fn test_clear_screen_error() {
        let std_io_error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let error = clearscreen::Error::Io(std_io_error);
        let io_error = Error::from(error);

        assert!(io_error.to_string().contains("test"));
    }

    #[test]
    fn test_parse_int_error() {
        let error = u64::from_str("foo").expect_err("expected ParseIntError");
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "invalid digit found in string");
    }

    #[test]
    fn test_std_io_error() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let io_error = Error::from(error);

        assert_eq!(io_error.to_string(), "test");
    }
}
