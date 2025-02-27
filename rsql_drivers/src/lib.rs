#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod driver;
mod error;
#[cfg(feature = "file")]
mod file;
#[cfg(feature = "http")]
mod http;
#[cfg(feature = "https")]
mod https;

pub use driver::DriverManager;
pub use error::{Error, Result};
pub use rsql_driver::{
    Column, Connection, Driver, Index, LimitQueryResult, MemoryQueryResult, Metadata,
    MockConnection, MockDriver, QueryResult, Row, Schema, StatementMetadata, Table, Value,
};
