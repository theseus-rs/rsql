#![cfg_attr(
    test,
    expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic when verification fails"
    )
)]

#[macro_use]
extern crate rust_i18n;

pub mod commands;
pub mod executors;
pub mod shell;

pub use rsql_formatters::writers;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
