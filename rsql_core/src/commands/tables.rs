use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use tracing::instrument;

/// List the tables in the database
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "tables"
    }

    fn description(&self) -> &'static str {
        "List the tables in the database"
    }

    #[instrument(name = "tables", skip(options))]
    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let output = options.output;
        let tables = options.connection.tables().await?;

        for table in tables {
            writeln!(output, "{}", table)?;
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
        let table = "table1";
        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_tables()
            .returning(|| Ok(vec![table.to_string()]));
        let mut output = Vec::new();
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".tables"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let tables = String::from_utf8(output)?;
        assert!(tables.contains(table));
        Ok(())
    }
}
