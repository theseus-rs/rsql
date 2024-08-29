#[derive(Debug, thiserror::Error)]
pub enum SnowflakeError {
    #[error("Unable to create snowflake client")]
    ClientCreation,
    #[error("JWT signature error")]
    JwtSignature,
    #[error("Unable to send Snowflake request: {0}")]
    Request(reqwest::Error),
    #[error("Snowflake response error: {0}")]
    Response(reqwest::Error),
    #[error("Malformed KEYPAIR_JWT private key")]
    MalformedPrivateKey,
    #[error("Malformed KEYPAIR_JWT public key")]
    MalformedPublicKey,
    #[error("Missing KEYPAIR_JWT private key")]
    MissingPrivateKey,
    #[error("Missing KEYPAIR_JWT public key")]
    MissingPublicKey,
    #[error("Missing account in connection string")]
    MissingAccount,
    #[error("Unable to create request headers")]
    MalformedHeaders,
    #[error("Unknown Snowflake Error")]
    Unspecified,
    #[error("Snowflake response content error: {0}")]
    ResponseContent(String),
}
