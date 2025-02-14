#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]

mod driver;
pub mod error;

pub use driver::Driver;
pub use error::SnowflakeError;
