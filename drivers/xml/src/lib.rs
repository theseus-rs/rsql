#![doc = include_str!("../README.md")]

mod driver;

pub use driver::Driver;
#[doc(hidden)]
pub use driver::xml_to_json;
