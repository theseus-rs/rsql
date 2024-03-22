use crate::commands::error::Result;
use crate::commands::{
    bail, clear, color, drivers, echo, exit, footer, format, header, help, history, locale, quit,
    tables, timer,
};
use crate::configuration::Configuration;
use crate::drivers::{Connection, DriverManager};
use crate::formatters::FormatterManager;
use async_trait::async_trait;
use rustyline::history::DefaultHistory;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io;

/// Loop condition for commands
///
/// `Continue`: Continue the loop
/// `Exit`: Exit the loop with the specified exit code
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoopCondition {
    Continue,
    Exit(i32),
}

/// Options for commands
pub struct CommandOptions<'a> {
    pub configuration: &'a mut Configuration,
    pub command_manager: &'a CommandManager,
    pub driver_manager: &'a DriverManager,
    pub formatter_manager: &'a FormatterManager,
    pub history: &'a DefaultHistory,
    pub connection: &'a mut dyn Connection,
    pub input: Vec<&'a str>,
    pub output: &'a mut (dyn io::Write + Send + Sync),
}

impl Debug for CommandOptions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandOptions")
            .field("configuration", &self.configuration)
            .field("command_manager", &self.command_manager)
            .field("driver_manager", &self.driver_manager)
            .field("formatter_manager", &self.formatter_manager)
            .field("connection", &self.connection)
            .field("input", &self.input)
            .finish()
    }
}

/// Trait that defines a command
#[async_trait]
pub trait ShellCommand: Debug + Sync {
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

/// Manages the active commands
#[derive(Debug)]
pub struct CommandManager {
    commands: BTreeMap<&'static str, Box<dyn ShellCommand>>,
}

impl CommandManager {
    /// Create a new instance of the `CommandManager` struct
    pub fn new() -> Self {
        CommandManager {
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

/// Default implementation for the `CommandManager` struct
impl Default for CommandManager {
    fn default() -> Self {
        let mut commands = CommandManager::new();

        commands.add(Box::new(bail::Command));
        commands.add(Box::new(clear::Command));
        commands.add(Box::new(color::Command));
        commands.add(Box::new(drivers::Command));
        commands.add(Box::new(echo::Command));
        commands.add(Box::new(exit::Command));
        commands.add(Box::new(footer::Command));
        commands.add(Box::new(format::Command));
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
    use crate::drivers::MockConnection;

    #[test]
    fn test_debug() {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &Default::default(),
            input: vec!["42"],
            output: &mut io::Cursor::new(Vec::new()),
        };

        let debug = format!("{:?}", options);
        assert!(debug.contains("CommandOptions"));
        assert!(debug.contains("configuration"));
        assert!(debug.contains("command_manager"));
        assert!(debug.contains("driver_manager"));
        assert!(debug.contains("formatter_manager"));
        assert!(debug.contains("connection"));
        assert!(debug.contains("input"));
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_commands() {
        let command = help::Command;
        let command_name = command.name();
        let mut command_manager = CommandManager::new();
        assert_eq!(command_manager.commands.len(), 0);

        command_manager.add(Box::new(help::Command));

        assert_eq!(command_manager.commands.len(), 1);
        let result = command_manager.get(command_name);
        assert!(result.is_some());

        let mut command_count = 0;
        command_manager.iter().for_each(|_command| {
            command_count += 1;
        });
        assert_eq!(command_count, 1);
    }

    #[test]
    fn test_command_manager_default() {
        let command_manager = CommandManager::default();

        assert_eq!(command_manager.commands.len(), 15);
    }
}
