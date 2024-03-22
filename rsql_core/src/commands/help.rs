extern crate colored;

use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use colored::*;

/// Show the help message
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "help"
    }

    fn description(&self) -> &'static str {
        "Show this help message"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let output = options.output;
        let command_identifier = &options.configuration.command_identifier;
        let command_manager = options.command_manager;
        let width = command_manager
            .iter()
            .map(|command| command_identifier.len() + command.name().len() + command.args().len())
            .max()
            .unwrap_or_default();

        for command in command_manager.iter() {
            let name = command.name();
            let arg_width = width - name.len();
            let args = if command.args().is_empty() {
                format!("{:arg_width$}", command.args(), arg_width = arg_width)
            } else {
                format!(" {:arg_width$}", command.args(), arg_width = arg_width - 1)
            };
            writeln!(
                output,
                "{}{}  {}",
                format!("{command_identifier}{name}").bold(),
                args.dimmed(),
                command.description(),
            )?;
        }
        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LoopCondition;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::configuration::Configuration;
    use crate::drivers::{DriverManager, MockConnection};
    use crate::formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    async fn test_execute(command_identifier: &str) -> anyhow::Result<()> {
        let mut configuration = Configuration {
            command_identifier: command_identifier.to_string(),
            ..Default::default()
        };
        let mut output = Vec::new();
        let command = format!("{command_identifier}help");
        let options = CommandOptions {
            configuration: &mut configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![command.as_str()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let help_output = String::from_utf8(output)?;
        assert!(help_output.contains(command.as_str()));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_postgresql_identifier() -> anyhow::Result<()> {
        test_execute("\\").await
    }

    #[tokio::test]
    async fn test_execute_sqlite_identifier() -> anyhow::Result<()> {
        test_execute(".").await
    }
}
