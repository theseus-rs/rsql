#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]

mod driver;
pub(crate) mod metadata;

pub use driver::{Connection, Driver};
