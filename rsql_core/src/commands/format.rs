use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to set the results format
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("format_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("format_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("format_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let formatter_manager = options.formatter_manager;

        if options.input.len() <= 1 {
            let format_setting = t!(
                "format_setting",
                locale = locale,
                format = options.configuration.results_format
            )
            .to_string();
            writeln!(options.output, "{}", format_setting)?;

            let list_delimiter = t!("list_delimiter", locale = locale).to_string();
            let formats: String = formatter_manager
                .iter()
                .map(|driver| driver.identifier())
                .collect::<Vec<_>>()
                .join(list_delimiter.as_str());
            let format_options =
                t!("format_options", locale = locale, formats = formats).to_string();
            writeln!(options.output, "{}", format_options)?;

            return Ok(LoopCondition::Continue);
        }

        let formatter_identifier = options.input[1].to_lowercase();
        match formatter_manager.get(formatter_identifier.as_str()) {
            Some(_) => options.configuration.results_format = formatter_identifier,
            None => {
                return Err(InvalidOption {
                    command_name: self.name(locale).to_string(),
                    option: formatter_identifier.to_string(),
                })
            }
        };

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

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "format");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[format]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(
            description,
            "Format results in ascii, csv, html, json, jsonl, tsv, unicode, xml, yaml"
        );
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            results_format: "unicode".to_string(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".format"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let format_output = String::from_utf8(output)?;
        assert_eq!(
            format_output,
            "Format: unicode\nFormats: ascii, csv, html, json, jsonl, tsv, unicode, xml, yaml\n"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_ascii() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            results_format: "unicode".to_string(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".format", "ascii"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.results_format, "ascii".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_unicode() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            results_format: "ascii".to_string(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".format", "unicode"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.results_format, "unicode".to_string());
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
            input: vec![".format", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
