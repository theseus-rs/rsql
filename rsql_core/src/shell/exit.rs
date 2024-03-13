use crate::shell::command::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use tracing::info;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn args(&self) -> &'static str {
        "[code]"
    }

    fn description(&self) -> &'static str {
        "Exit the application"
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
    use crate::configuration::Configuration;
    use crate::driver::MockConnection;
    use crate::shell::command::LoopCondition;
    use crate::shell::command::{CommandManager, CommandOptions};
    use rustyline::history::DefaultHistory;

    #[tokio::test]
    async fn test_execute_no_argument() -> Result<()> {
        let mock_connection = &mut MockConnection::new();
        mock_connection.expect_stop().returning(|| Ok(()));

        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
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
    async fn test_execute_argument() -> Result<()> {
        let mock_connection = &mut MockConnection::new();
        mock_connection.expect_stop().returning(|| Ok(()));

        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
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
    async fn test_execute_invalid() -> Result<()> {
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
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
