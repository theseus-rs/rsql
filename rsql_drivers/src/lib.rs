#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod driver_manager;

pub use driver_manager::DriverManager;
pub use rsql_driver::{
    Column, Connection, Driver, Error, Index, LimitQueryResult, MemoryQueryResult, Metadata,
    MockConnection, MockDriver, QueryResult, Result, Row, Schema, StatementMetadata, Table, Value,
};
