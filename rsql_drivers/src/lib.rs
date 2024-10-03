#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate rust_i18n;

mod connection;
mod driver;
#[cfg(feature = "duckdb")]
pub mod duckdb;
mod error;
#[cfg(feature = "libsql")]
pub mod libsql;
#[cfg(feature = "mariadb")]
pub mod mariadb;
mod metadata;
#[cfg(any(feature = "mariadb", feature = "mysql"))]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgresql")]
pub mod postgresql;
#[cfg(feature = "redshift")]
mod redshift;
mod row;
#[cfg(feature = "rusqlite")]
pub mod rusqlite;
#[cfg(feature = "snowflake")]
mod snowflake;
#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "sqlserver")]
mod sqlserver;
mod value;

pub use connection::{
    Connection, LimitQueryResult, MemoryQueryResult, MockConnection, QueryResult, StatementMetadata,
};
pub use driver::{Driver, DriverManager, MockDriver};
pub use error::{Error, Result};
pub use metadata::{Column, Index, Metadata, Schema, Table};
pub use row::Row;
pub use value::Value;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
