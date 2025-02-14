#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod driver;
mod error;

pub use driver::DriverManager;
pub use error::{Error, Result};
pub use rsql_driver::{
    Column, Connection, Driver, Index, LimitQueryResult, MemoryQueryResult, Metadata, QueryResult,
    Row, Schema, StatementMetadata, Table, Value,
};
