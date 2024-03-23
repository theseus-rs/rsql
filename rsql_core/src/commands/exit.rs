use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;
use tracing::info;

/// Exit the application
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("exit_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("exit_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("exit_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let exit_code = if options.input.len() == 1 {
            0
        } else {
            options.input[1].parse()?
        };

        options.connection.stop().await?;
        info!("Exiting with code {exit_code}");
        Ok(LoopCondition::Exit(exit_code))
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

    #[tokio::test]
    async fn test_execute_no_argument() -> anyhow::Result<()> {
        let mock_connection = &mut MockConnection::new();
        mock_connection.expect_stop().returning(|| Ok(()));

        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".exit"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Exit(0));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_argument() -> anyhow::Result<()> {
        let mock_connection = &mut MockConnection::new();
        mock_connection.expect_stop().returning(|| Ok(()));

        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".exit", "1"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Exit(1));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid() -> anyhow::Result<()> {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".exit", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
        Ok(())
    }
}
