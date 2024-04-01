#![forbid(unsafe_code)]
#[macro_use]
extern crate rust_i18n;

#[cfg(feature = "ascii")]
mod ascii;
#[cfg(feature = "csv")]
mod csv;
mod delimited;
mod error;
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
pub use formatter::{Formatter, FormatterManager, FormatterOptions};
pub use highlighter::Highlighter;

use rust_i18n::i18n;

i18n!("locales", fallback = "en");
