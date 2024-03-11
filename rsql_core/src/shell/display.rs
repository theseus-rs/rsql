use crate::configuration::ResultFormat;
use crate::shell::{CommandOptions, LoopCondition, Result, ShellCommand};
use anyhow::bail;
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "display"
    }

    fn args(&self) -> &'static str {
        "ascii|unicode"
    }

    fn description(&self) -> &'static str {
        "Display results in ASCII or Unicode"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let results_display = match options.input[1].to_lowercase().as_str() {
            "ascii" => ResultFormat::Ascii,
            "unicode" => ResultFormat::Unicode,
            option => bail!("Invalid display mode option: {option}"),
        };

        options.configuration.results_format = results_display;

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
    async fn test_execute_set_ascii() -> Result<()> {
        let configuration = &mut Configuration {
            results_format: ResultFormat::Unicode,
            ..default::Default::default()
        };
        let options = CommandOptions {
            input: vec![".display", "ascii"],
            configuration,
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.results_format, ResultFormat::Ascii);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_unicode() -> Result<()> {
        let configuration = &mut Configuration {
            results_format: ResultFormat::Ascii,
            ..default::Default::default()
        };
        let options = CommandOptions {
            input: vec![".display", "unicode"],
            configuration,
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.results_format, ResultFormat::Unicode);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            input: vec![".display", "foo"],
            configuration: &mut Configuration::default(),
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
