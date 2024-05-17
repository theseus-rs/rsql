use crate::commands::error::Result;
use crate::configuration::Configuration;
use async_trait::async_trait;
use rsql_drivers::{Connection, DriverManager};
use rsql_formatters::writers::Output;
use rsql_formatters::FormatterManager;
use rustyline::history::DefaultHistory;
use std::fmt::Debug;

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
    pub input: Vec<String>,
    pub output: &'a mut Output,
}

impl Debug for CommandOptions<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandOptions")
            .field("configuration", &self.configuration)
            .field("command_manager", &self.command_manager)
            .field("driver_manager", &self.driver_manager)
            .field("formatter_manager", &self.formatter_manager)
            .field("connection", &self.connection)
            .field("output", &self.output)
            .field("input", &self.input)
            .finish()
    }
}

/// Trait that defines a command
#[async_trait]
pub trait ShellCommand: Debug + Sync {
    /// Get the name of the command
    fn name(&self, locale: &str) -> String;
    /// Get the arguments for the command
    fn args(&self, _locale: &str) -> String {
        "".to_string()
    }
    /// Get the description of the command
    fn description(&self, locale: &str) -> String;
    /// Execute the command
    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition>;
}

/// Manages the active commands
#[derive(Debug)]
pub struct CommandManager {
    commands: Vec<Box<dyn ShellCommand>>,
}

impl CommandManager {
    /// Create a new instance of the `CommandManager` struct
    pub fn new() -> Self {
        CommandManager {
            commands: Vec::new(),
        }
    }

    /// Add a new command to the list of available commands
    pub fn add(&mut self, command: Box<dyn ShellCommand>) {
        let _ = &self.commands.push(command);
    }

    /// Get a command by name
    pub fn get(&self, locale: &str, name: &str) -> Option<&dyn ShellCommand> {
        for command in &self.commands {
            if command.name(locale) == name {
                return Some(command.as_ref());
            }
        }
        None
    }

    /// Get an iterator over the available commands
    pub(crate) fn iter(&self) -> impl Iterator<Item = &dyn ShellCommand> {
        self.commands.iter().map(|command| command.as_ref())
    }
}

/// Default implementation for the `CommandManager` struct
impl Default for CommandManager {
    fn default() -> Self {
        let mut commands = CommandManager::new();

        commands.add(Box::new(crate::commands::bail::Command));
        commands.add(Box::new(crate::commands::changes::Command));
        commands.add(Box::new(crate::commands::clear::Command));
        commands.add(Box::new(crate::commands::color::Command));
        commands.add(Box::new(crate::commands::describe::Command));
        commands.add(Box::new(crate::commands::drivers::Command));
        commands.add(Box::new(crate::commands::echo::Command));
        commands.add(Box::new(crate::commands::exit::Command));
        commands.add(Box::new(crate::commands::footer::Command));
        commands.add(Box::new(crate::commands::format::Command));
        commands.add(Box::new(crate::commands::header::Command));
        commands.add(Box::new(crate::commands::help::Command));
        commands.add(Box::new(crate::commands::history::Command));
        commands.add(Box::new(crate::commands::indexes::Command));
        commands.add(Box::new(crate::commands::limit::Command));
        commands.add(Box::new(crate::commands::locale::Command));
        commands.add(Box::new(crate::commands::output::Command));
        commands.add(Box::new(crate::commands::print::Command));
        commands.add(Box::new(crate::commands::quit::Command));
        commands.add(Box::new(crate::commands::read::Command));
        commands.add(Box::new(crate::commands::rows::Command));
        commands.add(Box::new(crate::commands::sleep::Command));
        commands.add(Box::new(crate::commands::system::Command));
        commands.add(Box::new(crate::commands::tables::Command));
        commands.add(Box::new(crate::commands::tee::Command));
        commands.add(Box::new(crate::commands::timer::Command));

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsql_drivers::MockConnection;

    #[test]
    fn test_debug() {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &Default::default(),
            input: vec!["42".to_string()],
            output: &mut Output::default(),
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
        let command = crate::commands::help::Command;
        let locale = "en";
        let command_name = command.name(locale);
        let mut command_manager = CommandManager::new();
        assert_eq!(command_manager.commands.len(), 0);

        command_manager.add(Box::new(command));

        assert_eq!(command_manager.commands.len(), 1);
        let result = command_manager.get(locale, command_name.as_str());
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

        assert_eq!(command_manager.commands.len(), 26);
    }
}
