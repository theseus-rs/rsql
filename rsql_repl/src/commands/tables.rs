use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_drivers::{MemoryQueryResult, Value};
use rsql_formatters::Results;
use rust_i18n::t;

/// List the tables in the schema
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("tables_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("tables_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let start = std::time::Instant::now();
        let output = options.output;
        let metadata = options.connection.metadata().await?;
        let configuration = options.configuration;
        let locale = &configuration.locale;
        let table_label = t!("table", locale = locale).to_string();
        let columns = vec![table_label];
        let mut rows = Vec::new();

        if let Some(catalog) = metadata.current_catalog() {
            if let Some(database) = catalog.current_schema() {
                let tables = database.tables();
                for table in tables {
                    let value = Value::String(table.name().to_string());
                    let row = vec![value];
                    rows.push(row);
                }
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        let mut results = Results::Query(Box::new(query_result));
        let formatter_options = &mut configuration.get_formatter_options();
        let result_format = &configuration.results_format;
        let formatter = options.formatter_manager.get(result_format).ok_or(
            rsql_formatters::Error::UnknownFormat {
                format: result_format.to_string(),
            },
        )?;

        formatter_options.elapsed = start.elapsed();
        formatter
            .format(formatter_options, &mut results, output)
            .await?;

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LoopCondition;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::writers::Output;
    use rsql_core::Configuration;
    use rsql_driver::Catalog;
    use rsql_drivers::{Metadata, MockConnection, Schema, Table};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "tables");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "List the tables in the schema");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut metadata = Metadata::new();
        let mut catalog = Catalog::new("default", true);
        let mut schema = Schema::new("default", true);
        let table_name = "table1";
        let table = Table::new(table_name);
        schema.add(table);
        catalog.add(schema);
        metadata.add(catalog);

        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_metadata()
            .returning(move || Ok(metadata.clone()));
        let mut output = Output::default();
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".tables".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let tables = output.to_string();
        assert!(tables.contains(table_name));
        Ok(())
    }
}
