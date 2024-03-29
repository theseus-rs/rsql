use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use colored::Colorize;
use num_format::{Locale, ToFormattedString};
use rust_i18n::t;
use std::str::FromStr;

/// Show the history of the shell
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("history_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();
        t!("on_off_argument", locale = locale, on = on, off = off).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("history_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();

        if options.input.len() <= 1 {
            let history = if options.configuration.history {
                for (i, entry) in options.history.iter().enumerate() {
                    let num_locale = Locale::from_str(locale).unwrap_or(Locale::en);
                    let index = (i + 1).to_formatted_string(&num_locale);
                    let mut entry = entry.to_string();
                    if options.configuration.color {
                        entry = entry.dimmed().to_string();
                    }

                    let history_list_entry = t!(
                        "history_list_entry",
                        locale = locale,
                        index = index,
                        entry = entry
                    );

                    writeln!(options.output, "{}", history_list_entry)?;
                }

                on
            } else {
                off
            };
            let history_setting =
                t!("history_setting", locale = locale, history = history).to_string();
            writeln!(options.output, "{}", history_setting)?;

            return Ok(LoopCondition::Continue);
        }

        let argument = options.input[1].to_lowercase().to_string();
        let history = if argument == on {
            true
        } else if argument == off {
            false
        } else {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: argument,
            });
        };

        options.configuration.history = history;

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
    use crate::writers::Output;
    use rustyline::history::{DefaultHistory, History};
    use std::default;
    use std::default::Default;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "history");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "on|off");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Show the command history");
    }

    #[tokio::test]
    async fn test_execute_history_enabled() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            history: true,
            ..Default::default()
        };
        let mut history = DefaultHistory::new();
        history.add("foo")?;

        let mut output = Output::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &history,
            input: vec![".history".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let history = output.to_string();
        assert!(history.contains("foo"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_history_disabled() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            history: false,
            ..Default::default()
        };
        let mut history = DefaultHistory::new();
        history.add("foo")?;

        let mut output = Output::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &history,
            input: vec![".history".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let history = output.to_string();
        assert!(!history.contains("foo"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_on() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            history: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".history".to_string(), "on".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.history);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            history: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".history".to_string(), "off".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.history);
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
            input: vec![".history".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
