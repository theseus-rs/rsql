extern crate colored;

use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use colored::*;

/// Show the help message
#[derive(Debug, Default)]
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
        let command_manager = options.command_manager;
        let width = command_manager
            .iter()
            .map(|command| command.name().len() + command.args().len() + 1)
            .max()
            .unwrap_or_default();

        for command in command_manager.iter() {
            let name = command.name();
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
    use crate::commands::LoopCondition;
    use crate::commands::{CommandManager, CommandOptions};
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
            input: vec![".help"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let help_output = String::from_utf8(output)?;
        assert!(help_output.contains(".help"));
        Ok(())
    }
}
