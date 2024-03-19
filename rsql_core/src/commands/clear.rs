use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use clearscreen::ClearScreen;

/// Clear the screen
#[derive(Debug, Default)]
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
        let clear_screen = ClearScreen::default();
        clear_screen.clear()?;
        Ok(LoopCondition::Continue)
    }
}
