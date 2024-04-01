#![forbid(unsafe_code)]
#[macro_use]
extern crate rust_i18n;

pub mod commands;
pub mod configuration;
pub mod executors;
pub mod shell;

pub use rsql_formatters::writers;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
