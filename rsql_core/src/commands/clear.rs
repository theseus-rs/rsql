use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Clear the screen
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("clear_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("clear_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, _options: CommandOptions<'a>) -> Result<LoopCondition> {
        clearscreen::clear()?;
        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "clear");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Clear the screen");
    }
}
