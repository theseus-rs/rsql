#![cfg_attr(
    test,
    expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic when verification fails"
    )
)]

mod driver_manager;

pub use driver_manager::DriverManager;
pub use rsql_driver::{
    Catalog, Column, Connection, Driver, Error, ForeignKey, Index, LimitQueryResult,
    MemoryQueryResult, Metadata, MockConnection, MockDriver, PrimaryKey, QueryResult, Result, Row,
    Schema, StatementMetadata, Table, ToSql, Value, ValueFormatter, View,
};
