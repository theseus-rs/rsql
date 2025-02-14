#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]

mod driver;
mod metadata;

pub use driver::{Connection, Driver};
pub use metadata::get_metadata;
