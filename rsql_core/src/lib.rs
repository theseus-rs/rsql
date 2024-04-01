#![forbid(unsafe_code)]
#[macro_use]
extern crate rust_i18n;

pub mod commands;
pub mod configuration;
pub mod executors;
pub mod formatters;
pub mod shell;
pub mod writers;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
