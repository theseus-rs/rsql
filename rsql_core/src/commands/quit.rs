use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;
use tracing::info;

/// Quit the application
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("quit_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("quit_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        options.connection.close().await?;

        info!("Quitting with code 0");
        Ok(LoopCondition::Exit(0))
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
        assert_eq!(name, "quit");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Quit the application");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mock_connection = &mut MockConnection::new();
        mock_connection.expect_close().returning(|| Ok(()));

        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".quit".to_string()],
            output: &mut Output::default(),
        };
        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Exit(0));
        Ok(())
    }
}
