use crate::shell::command::{CommandOptions, LoopCondition, Result, ShellCommand};
use anyhow::bail;
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "bail"
    }

    fn args(&self) -> &'static str {
        "on|off"
    }

    fn description(&self) -> &'static str {
        "Stop after an error occurs"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            let bail_on_error = if options.configuration.bail_on_error {
                "on"
            } else {
                "off"
            };
            writeln!(options.output, "Bail on error: {bail_on_error}")?;
            return Ok(LoopCondition::Continue);
        }

        let bail_on_error = match options.input[1].to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            option => bail!("Invalid bail option: {option}"),
        };

        options.configuration.bail_on_error = bail_on_error;

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::engine::MockEngine;
    use crate::shell::command::LoopCondition;
    use crate::shell::command::{CommandOptions, Commands};
    use rustyline::history::DefaultHistory;
    use std::default;

    #[tokio::test]
    async fn test_execute_no_args() -> Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            bail_on_error: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            commands: &Commands::default(),
            configuration,
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let bail_output = String::from_utf8(output)?;
        assert_eq!(bail_output, "Bail on error: on\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_on() -> Result<()> {
        let configuration = &mut Configuration {
            bail_on_error: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            commands: &Commands::default(),
            configuration,
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail", "on"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.bail_on_error);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> Result<()> {
        let configuration = &mut Configuration {
            bail_on_error: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            commands: &Commands::default(),
            configuration,
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail", "off"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.bail_on_error);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            commands: &Commands::default(),
            configuration: &mut Configuration::default(),
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            input: vec![".bail", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
