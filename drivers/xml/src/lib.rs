#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]

mod driver;

pub use driver::Driver;
#[doc(hidden)]
pub use driver::xml_to_json;
