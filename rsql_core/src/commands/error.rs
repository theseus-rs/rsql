use crate::commands::LoopCondition;

pub type Result<T = LoopCondition, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Driver error
    #[error(transparent)]
    DriverError(#[from] crate::drivers::Error),
    /// Error when an invalid option is provided for a command
    #[error("Invalid {command_name} option: {option}")]
    InvalidOption {
        command_name: String,
        option: String,
    },
    /// IO error
    #[error(transparent)]
    IoError(anyhow::Error),
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
