#![forbid(unsafe_code)]
#[macro_use]
extern crate rust_i18n;

mod connection;
mod driver;
mod error;
#[cfg(feature = "libsql")]
pub mod libsql;
#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgresql")]
pub mod postgresql;
#[cfg(feature = "rusqlite")]
pub mod rusqlite;
#[cfg(feature = "sqlite")]
pub mod sqlite;
mod value;

pub use connection::{Connection, MemoryQueryResult, MockConnection, QueryResult, Results};
pub use driver::{Driver, DriverManager, MockDriver};
pub use error::{Error, Result};
pub use value::Value;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
