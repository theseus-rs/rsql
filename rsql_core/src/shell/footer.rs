use crate::shell::command::{CommandOptions, LoopCondition, Result, ShellCommand};
use anyhow::bail;
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "footer"
    }

    fn args(&self) -> &'static str {
        "on|off"
    }

    fn description(&self) -> &'static str {
        "Enable or disable result footer"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            let footer = if options.configuration.results_footer {
                "on"
            } else {
                "off"
            };
            writeln!(options.output, "Footer: {footer}")?;
            return Ok(LoopCondition::Continue);
        }

        let footer = match options.input[1].to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            option => bail!("Invalid footer option: {option}"),
        };

        options.configuration.results_footer = footer;

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::driver::MockConnection;
    use crate::shell::command::LoopCondition;
    use crate::shell::command::{CommandManager, CommandOptions};
    use rustyline::history::DefaultHistory;
    use std::default;

    #[tokio::test]
    async fn test_execute_no_args() -> Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            results_footer: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".footer"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let footer_output = String::from_utf8(output)?;
        assert_eq!(footer_output, "Footer: on\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_on() -> Result<()> {
        let configuration = &mut Configuration {
            results_footer: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".footer", "on"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.results_footer);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> Result<()> {
        let configuration = &mut Configuration {
            results_footer: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".footer", "off"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.results_footer);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".footer", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
