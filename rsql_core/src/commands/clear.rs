use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use clearscreen::ClearScreen;
use std::io::Cursor;

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

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let clear_screen = ClearScreen::default();
        let mut output: Vec<u8> = Vec::new();
        let cursor = &mut Cursor::new(&mut output);
        clear_screen.clear_to(cursor)?;
        options.output.write_all(&output)?;
        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::commands::{CommandManager, CommandOptions, LoopCondition};
    use crate::configuration::Configuration;
    use crate::drivers::MockConnection;
    use rustyline::history::DefaultHistory;

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut output = Vec::new();
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".clear"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let output = String::from_utf8(output)?;
        assert!(!output.is_empty());
        Ok(())
    }
}
