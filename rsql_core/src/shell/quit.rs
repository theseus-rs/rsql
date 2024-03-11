use crate::shell::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use tracing::info;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "quit"
    }

    fn description(&self) -> &'static str {
        "Quit the application"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        options.engine.stop().await?;

        info!("Quitting with code 0");
        Ok(LoopCondition::Exit(0))
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

    #[tokio::test]
    async fn test_execute() -> Result<()> {
        let mock_engine = &mut MockEngine::new();
        mock_engine.expect_stop().returning(|| Ok(()));

        let options = CommandOptions {
            input: vec![".quit"],
            configuration: &mut Configuration::default(),
            engine: mock_engine,
            history: &DefaultHistory::new(),
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Exit(0));
        Ok(())
    }
}
