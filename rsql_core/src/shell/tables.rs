use crate::shell::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "tables"
    }

    fn description(&self) -> &'static str {
        "List the tables in the database"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let output = options.output;
        let tables = options.engine.tables().await?;

        for table in tables {
            writeln!(output, "{}", table)?;
        }

        Ok(LoopCondition::Continue)
    }
}
