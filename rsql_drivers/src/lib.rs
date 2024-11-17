#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate rust_i18n;

#[cfg(feature = "cockroachdb")]
mod cockroachdb;
mod connection;
mod driver;
#[cfg(feature = "duckdb")]
mod duckdb;
mod error;
#[cfg(feature = "libsql")]
mod libsql;
#[cfg(feature = "mariadb")]
mod mariadb;
mod metadata;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgresql")]
mod postgresql;
#[cfg(feature = "redshift")]
mod redshift;
#[cfg(feature = "rusqlite")]
mod rusqlite;
#[cfg(feature = "snowflake")]
mod snowflake;
#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlserver")]
mod sqlserver;
mod value;

pub use connection::{
    Connection, LimitQueryResult, MemoryQueryResult, MockConnection, QueryResult, StatementMetadata,
};
pub use driver::{Driver, DriverManager, MockDriver};
pub use error::{Error, Result};
pub use metadata::{Column, Index, Metadata, Schema, Table};
pub use value::Value;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
