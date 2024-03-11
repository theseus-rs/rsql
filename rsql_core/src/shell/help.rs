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
