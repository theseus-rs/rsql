use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to set the display locale
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("locale_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("locale_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("locale_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            writeln!(options.output, "Locale: {}", options.configuration.locale)?;
            return Ok(LoopCondition::Continue);
        }

        let locale = options.configuration.locale.as_str();
        let new_locale = options.input[1];

        if !available_locales!().contains(&new_locale) {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: locale.to_string(),
            });
        }
        options.configuration.locale = new_locale.to_string();

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

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            locale: "en".to_string(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".locale"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let locale_output = String::from_utf8(output)?;
        assert_eq!(locale_output, "Locale: en\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_locale() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            locale: "en".to_string(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".locale", "en-GB"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.locale, "en-GB".to_string());
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
            input: vec![".locale", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
