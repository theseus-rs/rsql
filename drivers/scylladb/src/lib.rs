//! Native CQL driver for `ScyllaDB` and Scylla Cloud.
//!
//! See the crate README for connection URL, TLS, and Client Routes configuration.

#![cfg_attr(
    test,
    expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic when verification fails"
    )
)]

mod config;
mod connection;
mod convert;
mod metadata;

pub use connection::{Connection, Driver};
