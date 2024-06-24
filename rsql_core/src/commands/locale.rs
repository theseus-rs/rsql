use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to set the display locale
#[derive(Debug, Default)]
pub struct Command;

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
        let locale = options.configuration.locale.as_str();

        if options.input.len() <= 1 {
            let format_setting = t!("locale_setting", locale = locale, locale = locale).to_string();
            writeln!(options.output, "{format_setting}")?;
            return Ok(LoopCondition::Continue);
        }

        let new_locale = options.input[1].as_str();

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
    use crate::writers::Output;
    use rsql_drivers::{DriverManager, MockConnection};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;
    use std::default;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "locale");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[locale]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Set the display locale");
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let mut output = Output::default();
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
            input: vec![".locale".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let locale_output = output.to_string();
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
            input: vec![".locale".to_string(), "en-GB".to_string()],
            output: &mut Output::default(),
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
            input: vec![".locale".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
