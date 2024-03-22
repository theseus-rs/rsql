use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;

/// Command to enable or disable echoing commands
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn args(&self) -> &'static str {
        "on|off"
    }

    fn description(&self) -> &'static str {
        "Enable or disable echoing commands"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            let echo = if options.configuration.echo {
                "on"
            } else {
                "off"
            };
            writeln!(options.output, "Echo: {echo}")?;
            return Ok(LoopCondition::Continue);
        }

        let echo = match options.input[1].to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            option => {
                return Err(InvalidOption {
                    command_name: self.name().to_string(),
                    option: option.to_string(),
                })
            }
        };

        options.configuration.echo = echo;

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
    use std::default;

    async fn test_execute_no_args(echo: bool) -> anyhow::Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            echo,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".echo"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let echo_output = String::from_utf8(output)?;

        if echo {
            assert_eq!(echo_output, "Echo: on\n");
        } else {
            assert_eq!(echo_output, "Echo: off\n");
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
            echo: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".echo", "on"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.echo);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            echo: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".echo", "off"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.echo);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".echo", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
