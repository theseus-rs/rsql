//! JDBC connections executed by the embedded Ristretto JVM.
#![cfg_attr(
    test,
    expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic when verification fails"
    )
)]

#[cfg(not(target_family = "wasm"))]
mod cache;
mod connection;
mod driver;
mod jdbc_type;
mod metadata;
mod options;
mod results;
mod values;

pub use connection::Connection;
pub use driver::Driver;
pub use options::Options;
