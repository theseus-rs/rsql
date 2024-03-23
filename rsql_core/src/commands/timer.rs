use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to enable or disable query execution timer
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("timer_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();
        t!("on_off_argument", locale = locale, on = on, off = off).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("timer_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();

        if options.input.len() <= 1 {
            let timer = if options.configuration.results_timer {
                on
            } else {
                off
            };
            let timer_setting = t!("timer_setting", locale = locale, timer = timer).to_string();
            writeln!(options.output, "{}", timer_setting)?;
            return Ok(LoopCondition::Continue);
        }

        let argument = options.input[1].to_lowercase().to_string();
        let timer = if argument == on {
            true
        } else if argument == off {
            false
        } else {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: argument,
            });
        };

        options.configuration.results_timer = timer;

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

    async fn test_execute_no_args(timer: bool) -> anyhow::Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            results_timer: timer,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".timer"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let timer_output = String::from_utf8(output)?;
        if timer {
            assert_eq!(timer_output, "Timer: on\n");
        } else {
            assert_eq!(timer_output, "Timer: off\n");
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
            results_timer: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".timer", "on"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.results_timer);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            results_timer: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".timer", "off"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.results_timer);
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
            input: vec![".timer", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
