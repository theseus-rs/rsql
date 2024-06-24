use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to print the specified string to the output.
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("print_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("print_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("print_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            writeln!(options.output)?;
        } else {
            for option in options.input.iter().skip(1) {
                writeln!(options.output, "{option}")?;
            }
        }

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
        assert_eq!(name, "print");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[string]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Print the specified string");
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            locale: "en".to_string(),
            ..default::Default::default()
        };
        let mut output = Output::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".print".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let locale_output = output.to_string();
        assert_eq!(locale_output, "\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_argument() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            locale: "en".to_string(),
            ..default::Default::default()
        };
        let mut output = Output::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".print".to_string(), "foo".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let locale_output = output.to_string();
        assert_eq!(locale_output, "foo\n");
        Ok(())
    }
}
