use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use crate::formatters::FormatterManager;
use anyhow::bail;
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "format"
    }

    fn args(&self) -> &'static str {
        "ascii|unicode"
    }

    fn description(&self) -> &'static str {
        "format results in ASCII or Unicode"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            writeln!(
                options.output,
                "Format: {}",
                options.configuration.results_format
            )?;
            return Ok(LoopCondition::Continue);
        }

        let formatter_manager = FormatterManager::default();
        let formatter_identifier = options.input[1].to_lowercase();
        match formatter_manager.get(formatter_identifier.as_str()) {
            Some(_) => options.configuration.results_format = formatter_identifier,
            None => bail!("Invalid format mode option: {formatter_identifier}"),
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
    use crate::drivers::MockConnection;
    use rustyline::history::DefaultHistory;
    use std::default;

    #[tokio::test]
    async fn test_execute_no_args() -> Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            results_format: "unicode".to_string(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".format"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let format_output = String::from_utf8(output)?;
        assert_eq!(format_output, "Format: unicode\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_ascii() -> Result<()> {
        let configuration = &mut Configuration {
            results_format: "unicode".to_string(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
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
    async fn test_execute_set_unicode() -> Result<()> {
        let configuration = &mut Configuration {
            results_format: "ascii".to_string(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
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
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".format", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}