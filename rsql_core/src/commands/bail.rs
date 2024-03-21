use async_trait::async_trait;

use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};

/// Command to stop after an error occurs
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "bail"
    }

    fn args(&self) -> &'static str {
        "on|off"
    }

    fn description(&self) -> &'static str {
        "Stop after an error occurs"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            let bail_on_error = if options.configuration.bail_on_error {
                "on"
            } else {
                "off"
            };
            writeln!(options.output, "Bail on error: {bail_on_error}")?;
            return Ok(LoopCondition::Continue);
        }

        let bail_on_error = match options.input[1].to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            option => {
                return Err(InvalidOption {
                    command_name: self.name().to_string(),
                    option: option.to_string(),
                })
            }
        };

        options.configuration.bail_on_error = bail_on_error;

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use std::default;

    use rustyline::history::DefaultHistory;

    use crate::commands::{CommandManager, CommandOptions, LoopCondition};
    use crate::configuration::Configuration;
    use crate::drivers::MockConnection;

    use super::*;

    async fn test_execute_no_args(bail: bool) -> anyhow::Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            bail_on_error: bail,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let bail_output = String::from_utf8(output)?;

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
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail", "on"],
            output: &mut Vec::new(),
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
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail", "off"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.bail_on_error);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
