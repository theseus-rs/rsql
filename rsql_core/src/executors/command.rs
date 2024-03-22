use crate::commands::{CommandManager, CommandOptions, LoopCondition};
use crate::configuration::Configuration;
use crate::drivers::{Connection, DriverManager};
use crate::executors::{Error, Result};
use crate::formatters::FormatterManager;
use rustyline::history::DefaultHistory;
use std::fmt::Debug;
use std::{fmt, io};

/// A command executor.
pub(crate) struct CommandExecutor<'a> {
    configuration: &'a mut Configuration,
    command_manager: &'a CommandManager,
    driver_manager: &'a DriverManager,
    formatter_manager: &'a FormatterManager,
    history: &'a DefaultHistory,
    connection: &'a mut dyn Connection,
    output: &'a mut (dyn io::Write + Send + Sync),
}

/// Implementation for [CommandExecutor].
impl<'a> CommandExecutor<'a> {
    pub(crate) fn new(
        configuration: &'a mut Configuration,
        command_manager: &'a CommandManager,
        driver_manager: &'a DriverManager,
        formatter_manager: &'a FormatterManager,
        history: &'a DefaultHistory,
        connection: &'a mut dyn Connection,
        output: &'a mut (dyn io::Write + Send + Sync),
    ) -> CommandExecutor<'a> {
        Self {
            configuration,
            command_manager,
            driver_manager,
            formatter_manager,
            history,
            connection,
            output,
        }
    }

    /// Execute the command and return the loop condition.
    pub(crate) async fn execute(&mut self, command: &str) -> Result<LoopCondition> {
        let input: Vec<&str> = command.split_whitespace().collect();
        let command_name = &input[0][1..input[0].len()];

        let loop_condition = match &self.command_manager.get(command_name) {
            Some(command) => {
                let options = CommandOptions {
                    configuration: self.configuration,
                    command_manager: self.command_manager,
                    driver_manager: self.driver_manager,
                    formatter_manager: self.formatter_manager,
                    connection: self.connection,
                    history: self.history,
                    input,
                    output: &mut self.output,
                };
                command.execute(options).await?
            }
            None => {
                return Err(Error::InvalidCommand {
                    command_name: command_name.to_string(),
                });
            }
        };

        Ok(loop_condition)
    }
}

impl Debug for CommandExecutor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandExecutor")
            .field("configuration", &self.configuration)
            .field("command_manager", &self.command_manager)
            .field("driver_manager", &self.driver_manager)
            .field("formatter_manager", &self.formatter_manager)
            .field("connection", &self.connection)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::MockConnection;

    #[tokio::test]
    async fn test_debug() {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let output = &mut io::stdout();

        let executor = CommandExecutor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            output,
        );
        let debug = format!("{:?}", executor);
        assert!(debug.contains("CommandExecutor"));
        assert!(debug.contains("configuration"));
        assert!(debug.contains("command_manager"));
        assert!(debug.contains("driver_manager"));
        assert!(debug.contains("formatter_manager"));
        assert!(debug.contains("connection"));
    }

    #[tokio::test]
    async fn test_execute_invalid_command() {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let output = &mut io::stdout();

        let mut executor = CommandExecutor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            output,
        );

        let result = executor.execute("foo").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            bail_on_error: false,
            ..Default::default()
        };
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let output = &mut io::stdout();

        let mut executor = CommandExecutor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            output,
        );

        let result = executor.execute(".bail on").await?;
        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.bail_on_error, true);
        Ok(())
    }
}
