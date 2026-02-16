//! # RSQL Driver
//!
//! The RSQL driver library provides interfaces for connecting to different data
//! sources and executing SQL queries.

#[macro_use]
extern crate rust_i18n;

mod connection;
mod driver;
mod driver_manager;
mod error;
mod metadata;
mod to_sql;
mod url;
mod value;

pub use connection::{
    CachedMetadataConnection, Connection, LimitQueryResult, MemoryQueryResult, MockConnection,
    QueryResult, Row, StatementMetadata, convert_to_at_placeholders,
    convert_to_numbered_placeholders,
};
pub use driver::{Driver, MockDriver};
pub use driver_manager::DriverManager;
pub use error::{Error, Result};
pub use metadata::{Catalog, Column, ForeignKey, Index, Metadata, PrimaryKey, Schema, Table};
pub use to_sql::{ToSql, to_values};
pub use url::UrlExtension;
pub use value::Value;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
