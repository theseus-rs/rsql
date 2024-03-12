extern crate colored;

mod clear;
mod command;
mod display;
mod exit;
mod footer;
mod header;
mod help;
mod history;
mod quit;
mod repl;
mod tables;
mod timer;

use clap::Parser;

pub use crate::shell::command::Commands;
pub use repl::execute;

#[derive(Debug, Parser)]
pub struct ShellArgs {
    /// The url of the database
    #[arg(long, default_value = "sqlite::memory:", env = "DATABASE_URL")]
    pub url: String,
}

impl Default for ShellArgs {
    fn default() -> Self {
        ShellArgs {
            url: "sqlite::memory:".to_string(),
        }
    }
}
