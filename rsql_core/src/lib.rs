#![forbid(unsafe_code)]

pub mod commands;
pub mod configuration;
pub mod drivers;
pub mod shell;
pub mod version;

use crate::shell::ShellArgs;
use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    /// The shell arguments
    #[clap(flatten)]
    pub shell_args: ShellArgs,

    /// Display the version of this tool
    #[arg(long)]
    version: bool,
}
