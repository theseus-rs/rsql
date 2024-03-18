mod ascii;
mod csv;
mod delimited;
mod error;
mod footer;
mod formatter;
mod html;
mod json;
mod jsonl;
mod table;
mod tsv;
mod unicode;
mod xml;
mod yaml;

pub use error::{Error, Result};
pub use formatter::{Formatter, FormatterManager, FormatterOptions};
