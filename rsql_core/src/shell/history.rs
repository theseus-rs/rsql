use crate::shell::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "history"
    }

    fn description(&self) -> &'static str {
        "Show the history of the shell"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let output = options.output;
        if options.configuration.history {
            for entry in options.history.iter() {
                writeln!(output, "{entry}")?;
            }
        } else {
            writeln!(output, "History is disabled")?;
        }
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
    use rustyline::history::{DefaultHistory, History};
    use std::default::Default;

    #[tokio::test]
    async fn test_execute_history_disabled() -> Result<()> {
        let configuration = &mut Configuration {
            history: false,
            ..Default::default()
        };
        let mut history = DefaultHistory::new();
        history.add("foo")?;

        let mut output = Vec::new();
        let options = CommandOptions {
            input: vec![".history"],
            configuration,
            engine: &mut MockEngine::new(),
            history: &history,
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let history = String::from_utf8(output)?;
        assert!(!history.contains("foo"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_history_enabled() -> Result<()> {
        let configuration = &mut Configuration {
            history: true,
            ..Default::default()
        };
        let mut history = DefaultHistory::new();
        history.add("foo")?;

        let mut output = Vec::new();
        let options = CommandOptions {
            input: vec![".history"],
            configuration,
            engine: &mut MockEngine::new(),
            history: &history,
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let history = String::from_utf8(output)?;
        assert!(history.contains("foo"));
        Ok(())
    }
}
