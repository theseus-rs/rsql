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

/// Converts a [`rustyline::error::ReadlineError`] into an [`IoError`](Error::IoError)
impl From<rustyline::error::ReadlineError> for Error {
    fn from(error: rustyline::error::ReadlineError) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`indicatif::style::TemplateError`] into an [`IoError`](Error::IoError)
impl From<indicatif::style::TemplateError> for Error {
    fn from(error: indicatif::style::TemplateError) -> Self {
        Error::IoError(error.into())
    }
}

/// Converts a [`std::io::Error`] into an [`IoError`](Error::IoError)
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error.into())
    }
}