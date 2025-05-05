#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate rust_i18n;

pub mod configuration;

pub use configuration::{Configuration, ConfigurationBuilder, EchoMode, EditMode};

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
