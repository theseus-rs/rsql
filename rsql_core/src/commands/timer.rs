use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use anyhow::bail;
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "timer"
    }

    fn args(&self) -> &'static str {
        "on|off"
    }

    fn description(&self) -> &'static str {
        "Enable or disable query execution timer"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            let timer = if options.configuration.results_timer {
                "on"
            } else {
                "off"
            };
            writeln!(options.output, "Timer: {timer}")?;
            return Ok(LoopCondition::Continue);
        }

        let timer = match options.input[1].to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            option => bail!("Invalid timing option: {option}"),
        };

        options.configuration.results_timer = timer;

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

    async fn test_execute_no_args(timer: bool) -> Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            results_timer: timer,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".timer"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let timer_output = String::from_utf8(output)?;
        if timer {
            assert_eq!(timer_output, "Timer: on\n");
        } else {
            assert_eq!(timer_output, "Timer: off\n");
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
            results_timer: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".timer", "on"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.results_timer);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> Result<()> {
        let configuration = &mut Configuration {
            results_timer: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".timer", "off"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.results_timer);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".timer", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
