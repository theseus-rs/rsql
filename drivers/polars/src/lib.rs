#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]

mod driver;
mod metadata;
mod value;

pub use driver::get_table_name;
pub use driver::Connection;
