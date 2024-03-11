extern crate colored;

use crate::shell::{CommandOptions, LoopCondition, Result, ShellCommand, COMMANDS};
use async_trait::async_trait;
use colored::*;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "help"
    }

    fn description(&self) -> &'static str {
        "Show this help message"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let output = options.output;
        let width = COMMANDS
            .iter()
            .map(|(name, command)| name.len() + command.args().len() + 1)
            .max()
            .unwrap_or_default();

        for (name, command) in COMMANDS.iter() {
            let arg_width = width - name.len();
            let args = if command.args().is_empty() {
                format!("{:arg_width$}", command.args(), arg_width = arg_width)
            } else {
                format!(" {:arg_width$}", command.args(), arg_width = arg_width - 1)
            };
            writeln!(
                output,
                "{}{}  {}",
                format!(".{name}").bold(),
                args.dimmed(),
                command.description(),
            )?;
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
    use rustyline::history::DefaultHistory;

    #[tokio::test]
    async fn test_execute() -> Result<()> {
        let mut output = Vec::new();
        let options = CommandOptions {
            input: vec![".help"],
            configuration: &mut Configuration::default(),
            engine: &mut MockEngine::new(),
            history: &DefaultHistory::new(),
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let help = String::from_utf8(output)?;
        assert!(help.contains(".help"));
        Ok(())
    }
}
