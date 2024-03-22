use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;

/// Show the history of the shell
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "history"
    }

    fn args(&self) -> &'static str {
        "on|off"
    }

    fn description(&self) -> &'static str {
        "Show the command history"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            let history = if options.configuration.history {
                for (i, entry) in options.history.iter().enumerate() {
                    writeln!(options.output, "{}: {entry}", i + 1)?;
                }

                "on"
            } else {
                "off"
            };
            writeln!(options.output, "History: {history}")?;

            return Ok(LoopCondition::Continue);
        }

        let history = match options.input[1].to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            option => {
                return Err(InvalidOption {
                    command_name: self.name().to_string(),
                    option: option.to_string(),
                })
            }
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
    use rustyline::history::{DefaultHistory, History};
    use std::default;
    use std::default::Default;

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
            input: vec![".history", "on"],
            output: &mut Vec::new(),
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
            input: vec![".history", "off"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.history);
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

        let mut output = Vec::new();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &history,
            input: vec![".history"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let history = String::from_utf8(output)?;
        assert!(!history.contains("foo"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_history_enabled() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            history: true,
            ..Default::default()
        };
        let mut history = DefaultHistory::new();
        history.add("foo")?;

        let mut output = Vec::new();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &history,
            input: vec![".history"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let history = String::from_utf8(output)?;
        assert!(history.contains("foo"));
        Ok(())
    }
}
