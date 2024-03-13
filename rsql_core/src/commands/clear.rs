use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn description(&self) -> &'static str {
        "Clear the screen"
    }

    async fn execute<'a>(&self, _options: CommandOptions<'a>) -> Result<LoopCondition> {
        clearscreen::clear()?;
        Ok(LoopCondition::Continue)
    }
}
