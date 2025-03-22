use async_trait::async_trait;

use crate::commands::{CommandOptions, ToggleShellCommand};

/// Command to stop after an error occurs
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ToggleShellCommand for Command {
    fn get_name(&self) -> &'static str {
        "bail_command"
    }

    fn get_description(&self) -> &'static str {
        "bail_description"
    }

    fn get_setting_str(&self) -> &'static str {
        "bail_setting"
    }

    fn get_value(&self, options: &CommandOptions<'_>) -> bool {
        options.configuration.bail_on_error
    }

    fn set_value(&self, options: &mut CommandOptions<'_>, value: bool) {
        options.configuration.bail_on_error = value;
    }
}

#[cfg(test)]
mod tests {
    use std::default;

    use rustyline::history::DefaultHistory;

    use crate::commands::{CommandManager, CommandOptions, LoopCondition, ShellCommand};
    use crate::configuration::Configuration;
    use crate::writers::Output;
    use rsql_drivers::MockConnection;
    use rsql_formatters::FormatterManager;

    use super::*;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "bail");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "on|off");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Stop after an error occurs");
    }

    async fn test_execute_no_args(bail: bool) -> anyhow::Result<()> {
        let mut output = Output::default();
        let configuration = &mut Configuration {
            bail_on_error: bail,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let bail_output = output.to_string();

        if bail {
            assert_eq!(bail_output, "Bail on error: on\n");
        } else {
            assert_eq!(bail_output, "Bail on error: off\n");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_no_args_on() -> anyhow::Result<()> {
        test_execute_no_args(true).await
    }

    #[tokio::test]
    async fn test_execute_no_args_off() -> anyhow::Result<()> {
        test_execute_no_args(false).await
    }

    #[tokio::test]
    async fn test_execute_set_on() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            bail_on_error: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail".to_string(), "on".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.bail_on_error);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            bail_on_error: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail".to_string(), "off".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.bail_on_error);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
