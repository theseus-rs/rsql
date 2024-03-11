use crate::shell::{CommandOptions, LoopCondition, Result, ShellCommand};
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
    use crate::engine::MockEngine;
    use crate::shell::CommandOptions;
    use crate::shell::LoopCondition;
    use rustyline::history::DefaultHistory;
    use std::default;

    #[tokio::test]
    async fn test_execute_set_on() -> Result<()> {
        let configuration = &mut Configuration {
            results_footer: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            input: vec![".footer", "on"],
            configuration,
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
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
            input: vec![".footer", "off"],
            configuration,
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
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
            input: vec![".footer", "foo"],
            configuration: &mut Configuration::default(),
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
