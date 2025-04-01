#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate rust_i18n;

pub mod commands;
pub mod executors;
pub mod shell;

pub use rsql_formatters::writers;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
