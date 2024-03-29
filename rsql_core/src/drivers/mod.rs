mod connection;
mod driver;
mod error;
#[cfg(feature = "postgres")]
pub(crate) mod postgres;
#[cfg(feature = "postgresql")]
pub(crate) mod postgresql;
#[cfg(feature = "rusqlite")]
pub(crate) mod rusqlite;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;
mod value;

pub use connection::MemoryQueryResult;
#[cfg(test)]
pub(crate) use connection::MockConnection;
pub use connection::{Connection, QueryResult, Results};
#[cfg(test)]
pub(crate) use driver::MockDriver;
pub use driver::{Driver, DriverManager};
pub use error::{Error, Result};
pub use value::Value;
