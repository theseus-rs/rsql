#[cfg(feature = "postgresql")]
pub(crate) mod postgresql;

mod connection;
mod driver;
mod error;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;
mod value;

#[cfg(test)]
pub use connection::MockConnection;
pub use connection::{Connection, QueryResult, Results};
pub use driver::{Driver, DriverManager};
pub use error::{Error, Result};
pub use value::Value;
