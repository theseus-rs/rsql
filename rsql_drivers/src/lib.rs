#![forbid(unsafe_code)]
#![forbid(clippy::allow_attributes)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate rust_i18n;

#[cfg(feature = "arrow")]
mod arrow;
#[cfg(feature = "avro")]
mod avro;
#[cfg(feature = "cockroachdb")]
mod cockroachdb;
mod connection;
#[cfg(feature = "csv")]
mod csv;
#[cfg(feature = "delimited")]
mod delimited;
mod driver;
#[cfg(feature = "duckdb")]
mod duckdb;
mod error;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "jsonl")]
mod jsonl;
#[cfg(feature = "libsql")]
mod libsql;
#[cfg(feature = "mariadb")]
mod mariadb;
mod metadata;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "parquet")]
mod parquet;
#[cfg(any(
    feature = "arrow",
    feature = "avro",
    feature = "csv",
    feature = "delimited",
    feature = "json",
    feature = "jsonl",
    feature = "parquet",
    feature = "tsv"
))]
mod polars;
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
#[cfg(feature = "tsv")]
mod tsv;
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
