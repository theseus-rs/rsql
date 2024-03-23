use async_trait::async_trait;

use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use rust_i18n::t;

/// Command to stop after an error occurs
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("bail_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();
        t!("on_off_argument", locale = locale, on = on, off = off).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("bail_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();

        if options.input.len() <= 1 {
            let bail = if options.configuration.bail_on_error {
                on
            } else {
                off
            };
            let bail_setting = t!("bail_setting", locale = locale, bail = bail).to_string();
            writeln!(options.output, "{}", bail_setting)?;
            return Ok(LoopCondition::Continue);
        }

        let argument = options.input[1].to_lowercase().to_string();
        let bail_on_error = if argument == on {
            true
        } else if argument == off {
            false
        } else {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: argument,
            });
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
    use crate::drivers::{DriverManager, MockConnection};
    use crate::formatters::FormatterManager;

    use super::*;

    async fn test_execute_no_args(bail: bool) -> anyhow::Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            bail_on_error: bail,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
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
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
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
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
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
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
