use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum SnowflakeError {
    ClientCreation,
    JwtSignature,
    Request,
    Response,
    MissingPrivateKey,
    MissingAccount,
    MissingUser,
    MissingPublicKey,
    MalformedHeaders,
    MalformedPrivateKey,
    MalformedPublicKey,
    Unspecified,
    ResponseContent(String),
}

impl Error for SnowflakeError {}
impl Display for SnowflakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnowflakeError::ClientCreation => write!(f, "Unable to create snowflake client"),
            SnowflakeError::JwtSignature => write!(f, "JWT signature error"),
            SnowflakeError::Request => write!(f, "Request error"),
            SnowflakeError::Response => write!(f, "Response error"),
            SnowflakeError::MissingPrivateKey => write!(f, "Missing private key"),
            SnowflakeError::MissingAccount => write!(f, "Missing account"),
            SnowflakeError::MissingUser => write!(f, "Missing user"),
            SnowflakeError::MissingPublicKey => write!(f, "Missing public key"),
            SnowflakeError::MalformedHeaders => write!(f, "Unable to create request headers"),
            SnowflakeError::MalformedPrivateKey => write!(f, "Malformed private key"),
            SnowflakeError::MalformedPublicKey => write!(f, "Malformed public key"),
            SnowflakeError::Unspecified => write!(f, "Unspecified error"),
            SnowflakeError::ResponseContent(s) => write!(f, "Response content error: {s}"),
        }
    }
}
