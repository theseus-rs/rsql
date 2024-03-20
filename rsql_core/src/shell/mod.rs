extern crate colored;

mod args;
mod completer;
mod error;
mod helper;
mod highlighter;
mod repl;

pub use args::ShellArgs;
pub use error::{Error, Result};
pub use repl::{Shell, ShellBuilder};
