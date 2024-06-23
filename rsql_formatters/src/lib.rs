#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate rust_i18n;

#[cfg(feature = "ascii")]
mod ascii;
#[cfg(feature = "csv")]
mod csv;
#[cfg(any(feature = "csv", feature = "sqlite", feature = "tsv"))]
mod delimited;
mod error;
#[cfg(feature = "expanded")]
mod expanded;
mod footer;
mod formatter;
mod highlighter;
#[cfg(feature = "html")]
mod html;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "jsonl")]
mod jsonl;
#[cfg(feature = "markdown")]
mod markdown;
#[cfg(feature = "plain")]
mod plain;
#[cfg(feature = "psql")]
mod psql;
#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(any(
    feature = "ascii",
    feature = "markdown",
    feature = "plain",
    feature = "psql",
    feature = "unicode"
))]
mod table;
#[cfg(feature = "tsv")]
mod tsv;
#[cfg(feature = "unicode")]
mod unicode;
pub mod writers;
#[cfg(feature = "xml")]
mod xml;
#[cfg(feature = "yaml")]
mod yaml;

pub use error::{Error, Result};
pub use formatter::{Formatter, FormatterManager, FormatterOptions, Results};
pub use highlighter::Highlighter;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
