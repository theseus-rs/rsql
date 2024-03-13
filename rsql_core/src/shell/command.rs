use crate::configuration::Configuration;
use crate::engine::Engine;
use crate::shell::{
    bail, clear, display, exit, footer, header, help, history, locale, quit, tables, timer,
};
use async_trait::async_trait;
use rustyline::history::DefaultHistory;
use std::collections::BTreeMap;
use std::io;

/// Loop condition for shell commands
///
/// `Continue`: Continue the loop
/// `Exit`: Exit the loop with the specified exit code
#[derive(Debug, Eq, PartialEq)]
pub enum LoopCondition {
    Continue,
    Exit(i32),
}

/// Result type for shell commands
pub type Result<T = LoopCondition, E = anyhow::Error> = core::result::Result<T, E>;

/// Options for shell commands
pub struct CommandOptions<'a> {
    pub(crate) commands: &'a Commands,
    pub(crate) configuration: &'a mut Configuration,
    pub(crate) engine: &'a mut dyn Engine,
    pub(crate) history: &'a DefaultHistory,
    pub(crate) input: Vec<&'a str>,
    pub(crate) output: &'a mut (dyn io::Write + Send),
}

/// Trait that defines a shell command
#[async_trait]
pub trait ShellCommand: Sync {
    /// Get the name of the command
    fn name(&self) -> &'static str;
    /// Get the arguments for the command
    fn args(&self) -> &'static str {
        ""
    }
    /// Get the description of the command
    fn description(&self) -> &'static str;
    /// Execute the command
    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition>;
}

/// Manages the active shell commands
pub struct Commands {
    commands: BTreeMap<&'static str, Box<dyn ShellCommand>>,
}

impl Commands {
    /// Create a new instance of the `Commands` struct
    pub fn new() -> Self {
        Commands {
            commands: BTreeMap::new(),
        }
    }

    /// Add a new command to the list of available commands
    fn add(&mut self, command: Box<dyn ShellCommand>) {
        let name = command.name();
        let _ = &self.commands.insert(name, command);
    }

    /// Get a command by name
    pub fn get(&self, name: &str) -> Option<&dyn ShellCommand> {
        self.commands.get(name).map(|command| command.as_ref())
    }

    /// Get an iterator over the available commands
    pub(crate) fn iter(&self) -> impl Iterator<Item = &dyn ShellCommand> {
        self.commands.values().map(|command| command.as_ref())
    }
}

/// Default implementation for the `Commands` struct
// .autocomplete on|off      Enable or disable auto-completion
// .multi on|off             Enable or disable multiline mode
// .output [mode] [options]  Set output format: csv, json, table or line
impl Default for Commands {
    fn default() -> Self {
        let mut commands = Commands::new();

        commands.add(Box::new(bail::Command));
        commands.add(Box::new(clear::Command));
        commands.add(Box::new(display::Command));
        commands.add(Box::new(exit::Command));
        commands.add(Box::new(footer::Command));
        commands.add(Box::new(header::Command));
        commands.add(Box::new(help::Command));
        commands.add(Box::new(history::Command));
        commands.add(Box::new(locale::Command));
        commands.add(Box::new(tables::Command));
        commands.add(Box::new(timer::Command));
        commands.add(Box::new(quit::Command));

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands() {
        let command = help::Command;
        let command_name = command.name();
        let mut commands = Commands::new();
        assert_eq!(commands.commands.len(), 0);

        commands.add(Box::new(help::Command));

        assert_eq!(commands.commands.len(), 1);
        let result = commands.get(command_name);
        assert!(result.is_some());

        let mut command_count = 0;
        commands.iter().for_each(|_command| {
            command_count += 1;
        });
        assert_eq!(command_count, 1);
    }

    #[test]
    fn test_commands_default() {
        let commands = Commands::default();

        assert_eq!(commands.commands.len(), 12);
    }
}
