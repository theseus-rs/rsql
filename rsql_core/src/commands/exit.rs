use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;
use tracing::info;

/// Exit the application
#[derive(Debug, Default)]
pub struct Command;

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

        options.connection.close().await?;
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
    use crate::writers::Output;
    use rsql_drivers::{DriverManager, MockConnection};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "exit");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[code]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Exit the application");
    }

    #[tokio::test]
    async fn test_execute_no_argument() -> anyhow::Result<()> {
        let mock_connection = &mut MockConnection::new();
        mock_connection.expect_close().returning(|| Ok(()));

        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".exit".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Exit(0));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_argument() -> anyhow::Result<()> {
        let mock_connection = &mut MockConnection::new();
        mock_connection.expect_close().returning(|| Ok(()));

        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".exit".to_string(), "1".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Exit(1));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid() {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".exit".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
