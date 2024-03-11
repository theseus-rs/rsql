use crate::shell::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use tracing::info;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn args(&self) -> &'static str {
        "[code]"
    }

    fn description(&self) -> &'static str {
        "Exit the application"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        options.engine.stop().await?;

        let exit_code = if options.input.len() == 1 {
            0
        } else {
            options.input[1].parse()?
        };

        info!("Exiting with code {exit_code}");
        Ok(LoopCondition::Exit(exit_code))
    }
}
