use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use crate::formatters::FormatterManager;
use async_trait::async_trait;

/// A shell command to set the results format
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "format"
    }

    fn args(&self) -> &'static str {
        "[format]"
    }

    fn description(&self) -> &'static str {
        "Format results in ascii, csv, json, jsonl, tsv, unicode, ..."
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let formatter_manager = FormatterManager::default();

        if options.input.len() <= 1 {
            writeln!(
                options.output,
                "Format: {}",
                options.configuration.results_format
            )?;

            write!(options.output, "Available formats: ")?;
            for (i, formatter) in formatter_manager.iter().enumerate() {
                if i > 0 {
                    write!(options.output, ", ")?;
                }
                write!(options.output, "{}", formatter.identifier())?;
            }
            writeln!(options.output)?;

            return Ok(LoopCondition::Continue);
        }

        let formatter_identifier = options.input[1].to_lowercase();
        match formatter_manager.get(formatter_identifier.as_str()) {
            Some(_) => options.configuration.results_format = formatter_identifier,
            None => {
                return Err(InvalidOption {
                    command_name: self.name().to_string(),
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
    use crate::drivers::MockConnection;
    use rustyline::history::DefaultHistory;
    use std::default;

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
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
        assert_eq!(
            format_output,
            "Format: unicode\nAvailable formats: ascii, csv, json, jsonl, tsv, unicode\n"
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
    async fn test_execute_set_unicode() -> anyhow::Result<()> {
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
