//! # RSQL Driver
//!
//! The RSQL driver library provides interfaces for connecting to different data
//! sources and executing SQL queries.

#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]

#[macro_use]
extern crate rust_i18n;

mod connection;
mod driver;
mod driver_manager;
mod error;
mod metadata;
mod url;
mod value;

pub use connection::{
    CachedMetadataConnection, Connection, LimitQueryResult, MemoryQueryResult, MockConnection,
    QueryResult, Row, StatementMetadata,
};
pub use driver::{Driver, MockDriver};
pub use driver_manager::DriverManager;
pub use error::{Error, Result};
pub use metadata::{Column, Index, Metadata, Schema, Table};
pub use url::UrlExtension;
pub use value::Value;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
