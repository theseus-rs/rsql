use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use num_format::{Locale, ToFormattedString};
use rust_i18n::t;
use std::str::FromStr;

/// Command to limit the number of rows returned by a query.
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("limit_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("limit_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("limit_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();

        if options.input.len() <= 1 {
            let num_locale = Locale::from_str(locale).unwrap_or(Locale::en);
            let limit = options
                .configuration
                .results_limit
                .to_formatted_string(&num_locale);
            let limit_setting = t!("limit_setting", locale = locale, limit = limit).to_string();
            writeln!(options.output, "{}", limit_setting)?;
            return Ok(LoopCondition::Continue);
        }

        options.configuration.results_limit = options.input[1].parse::<u64>()?;

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::configuration::Configuration;
    use crate::drivers::{DriverManager, MockConnection};
    use crate::formatters::FormatterManager;
    use crate::writers::Output;
    use rustyline::history::DefaultHistory;
    use std::default;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "limit");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[limit]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Set the maximum number of results to return");
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let mut output = Output::default();
        let configuration = &mut Configuration {
            results_limit: 1234,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".limit".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let limit_output = output.to_string();
        assert_eq!(limit_output, "Limit: 1,234\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_limit_500_millis() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            results_limit: 0,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration: configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".limit".to_string(), "42".to_string()],
            output: &mut Output::default(),
        };

        let _ = Command.execute(options).await?;

        assert_eq!(configuration.results_limit, 42);
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
            input: vec![".limit".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
