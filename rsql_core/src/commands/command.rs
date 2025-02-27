use crate::commands::Error::InvalidOption;
use crate::commands::error::Result;
use crate::configuration::Configuration;
use async_trait::async_trait;
use rsql_drivers::{Connection, DriverManager};
use rsql_formatters::FormatterManager;
use rsql_formatters::writers::Output;
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
        String::new()
    }
    /// Get the description of the command
    fn description(&self, locale: &str) -> String;
    /// Execute the command
    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition>;
}

#[async_trait]
pub trait ToggleShellCommand: Debug + Sync {
    fn get_value(&self, options: &CommandOptions<'_>) -> bool;
    fn set_value(&self, options: &mut CommandOptions<'_>, value: bool);

    fn get_name(&self) -> &'static str;
    fn get_description(&self) -> &'static str;
    fn get_setting_str(&self) -> &'static str;
}

#[async_trait]
impl<T: ToggleShellCommand> ShellCommand for T {
    fn name(&self, locale: &str) -> String {
        t!(self.get_name(), locale = locale).to_string()
    }
    fn args(&self, locale: &str) -> String {
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();
        t!("on_off_argument", locale = locale, on = on, off = off).to_string()
    }
    fn description(&self, locale: &str) -> String {
        t!(self.get_description(), locale = locale).to_string()
    }
    async fn execute<'a>(&self, mut options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();

        if options.input.len() <= 1 {
            let setting_enabled_text = if self.get_value(&options) { on } else { off };
            let setting = t!(
                self.get_setting_str(),
                locale = locale,
                setting = setting_enabled_text
            )
            .to_string();
            writeln!(options.output, "{setting}")?;
            return Ok(LoopCondition::Continue);
        }

        let argument = options.input[1].to_lowercase().to_string();
        let new_setting = if argument == on {
            true
        } else if argument == off {
            false
        } else {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: argument,
            });
        };

        self.set_value(&mut options, new_setting);

        Ok(LoopCondition::Continue)
    }
}

/// Manages the active commands
#[derive(Debug)]
pub struct CommandManager {
    commands: Vec<Box<dyn ShellCommand>>,
}

impl CommandManager {
    /// Create a new instance of the `CommandManager` struct
    #[must_use]
    pub fn new() -> Self {
        CommandManager {
            commands: Vec::new(),
        }
    }

    /// Add a new command to the list of available commands
    pub fn add(&mut self, command: Box<dyn ShellCommand>) {
        let () = &self.commands.push(command);
    }

    /// Get a command by name
    #[must_use]
    pub fn get(&self, locale: &str, name: &str) -> Option<&dyn ShellCommand> {
        for command in &self.commands {
            if command.name(locale) == name {
                return Some(command.as_ref());
            }
        }
        None
    }

    /// Gets a command by starts with prefix if it is unique
    #[must_use]
    pub fn get_starts_with(&self, locale: &str, prefix: &str) -> Option<&dyn ShellCommand> {
        let mut result: Option<&dyn ShellCommand> = None;
        for command in &self.commands {
            if command.name(locale).starts_with(prefix) {
                if result.is_some() {
                    return None;
                }
                result = Some(command.as_ref());
            }
        }

        result
    }

    /// Get an iterator over the available commands
    pub(crate) fn iter(&self) -> impl Iterator<Item = &dyn ShellCommand> {
        self.commands.iter().map(AsRef::as_ref)
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
        commands.add(Box::new(crate::commands::completions::Command));
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
        commands.add(Box::new(crate::commands::schemas::Command));
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
    use rustyline::history::FileHistory;

    #[test]
    fn test_debug() {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &FileHistory::default(),
            input: vec!["42".to_string()],
            output: &mut Output::default(),
        };

        let debug = format!("{options:?}");
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
    fn test_get_starts_with() {
        let command_manager = CommandManager::default();
        let locale = "en";
        let header_command = command_manager.get(locale, "header");
        let help_command = command_manager.get(locale, "help");

        assert!(header_command.is_some());
        assert!(help_command.is_some());

        let result = command_manager.get_starts_with(locale, "he");
        assert!(result.is_none());

        let result = command_manager.get_starts_with(locale, "head");
        assert!(result.is_some());
    }

    #[test]
    fn test_command_manager_default() {
        let command_manager = CommandManager::default();

        assert_eq!(command_manager.commands.len(), 28);
    }
}
