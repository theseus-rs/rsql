use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use anyhow::bail;
use async_trait::async_trait;

/// A shell command to enable or disable result header
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "header"
    }

    fn args(&self) -> &'static str {
        "on|off"
    }

    fn description(&self) -> &'static str {
        "Enable or disable result header"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            let header = if options.configuration.results_header {
                "on"
            } else {
                "off"
            };
            writeln!(options.output, "Header: {header}")?;
            return Ok(LoopCondition::Continue);
        }

        let header = match options.input[1].to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            option => bail!("Invalid header option: {option}"),
        };

        options.configuration.results_header = header;

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

    async fn test_execute_no_args(header: bool) -> Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            results_header: header,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".header"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let header_output = String::from_utf8(output)?;

        if header {
            assert_eq!(header_output, "Header: on\n");
        } else {
            assert_eq!(header_output, "Header: off\n");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_no_args_on() -> Result<()> {
        test_execute_no_args(true).await
    }

    #[tokio::test]
    async fn test_execute_no_args_off() -> Result<()> {
        test_execute_no_args(false).await
    }

    #[tokio::test]
    async fn test_execute_set_on() -> Result<()> {
        let configuration = &mut Configuration {
            results_header: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".header", "on"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.results_header);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> Result<()> {
        let configuration = &mut Configuration {
            results_header: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".header", "off"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.results_header);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".header", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
