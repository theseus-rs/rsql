use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to display index information.
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, index: &str) -> String {
        t!("indexes_command", index = index).to_string()
    }

    fn args(&self, index: &str) -> String {
        t!("indexes_argument", index = index).to_string()
    }

    fn description(&self, index: &str) -> String {
        t!("indexes_description", index = index).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let output = options.output;
        let table = options.input.get(1).map(|s| s.as_str());
        let indexes = options.connection.indexes(table).await?;

        for index in indexes {
            writeln!(output, "{}", index)?;
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
    use crate::writers::Output;
    use rsql_drivers::{DriverManager, MockConnection};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "indexes");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[table]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Display the indexes");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let index = "index1";
        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_indexes()
            .returning(|_| Ok(vec![index.to_string()]));
        let mut output = Output::default();
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".indexes".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let tables = output.to_string();
        assert!(tables.contains(index));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_table() -> anyhow::Result<()> {
        let table = "table1";
        let index = "index1";
        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_indexes()
            .returning(|_| Ok(vec![index.to_string()]));
        let mut output = Output::default();
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".indexes".to_string(), table.to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let tables = output.to_string();
        assert!(tables.contains(index));
        Ok(())
    }
}
