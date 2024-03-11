extern crate colored;

mod clear;
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

use crate::engine::Engine;
use async_trait::async_trait;
use clap::Parser;
use lazy_static::lazy_static;
use rustyline::history::DefaultHistory;
use std::collections::BTreeMap;
use std::io;

use crate::configuration::Configuration;
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

#[derive(Debug, Eq, PartialEq)]
pub enum LoopCondition {
    Continue,
    Exit(i32),
}

pub type Result<T = LoopCondition, E = anyhow::Error> = core::result::Result<T, E>;

pub struct CommandOptions<'a> {
    pub(crate) configuration: &'a mut Configuration,
    pub(crate) engine: &'a mut dyn Engine,
    pub(crate) history: &'a DefaultHistory,
    pub(crate) input: Vec<&'a str>,
    pub(crate) output: &'a mut (dyn io::Write + Send),
}

#[async_trait]
pub trait ShellCommand: Sync {
    fn name(&self) -> &'static str;
    fn args(&self) -> &'static str {
        ""
    }
    fn description(&self) -> &'static str;
    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition>;
}

// .autocomplete on|off      Enable or disable auto-completion
// .multi on|off             Enable or disable multiline mode
// .output [mode] [options]  Set output format: csv, json, table or line
lazy_static! {
    static ref COMMANDS: BTreeMap<&'static str, Box<dyn ShellCommand>> = {
        let mut map: BTreeMap<&'static str, Box<dyn ShellCommand>> = BTreeMap::new();

        insert_command(&mut map, Box::new(clear::Command));
        insert_command(&mut map, Box::new(display::Command));
        insert_command(&mut map, Box::new(exit::Command));
        insert_command(&mut map, Box::new(footer::Command));
        insert_command(&mut map, Box::new(header::Command));
        insert_command(&mut map, Box::new(help::Command));
        insert_command(&mut map, Box::new(history::Command));
        insert_command(&mut map, Box::new(tables::Command));
        insert_command(&mut map, Box::new(timer::Command));
        insert_command(&mut map, Box::new(quit::Command));

        map
    };
}

fn insert_command(
    commands: &mut BTreeMap<&'static str, Box<dyn ShellCommand>>,
    command: Box<dyn ShellCommand>,
) {
    let name = command.name();

    commands.insert(name, command);
}

pub(crate) fn get_command(command_name: &str) -> Option<&dyn ShellCommand> {
    let configuration = COMMANDS.get(command_name);
    match configuration {
        Some(command) => Some(command.as_ref()),
        None => None,
    }
}
