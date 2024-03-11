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
