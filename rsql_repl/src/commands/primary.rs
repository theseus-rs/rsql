use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_drivers::{MemoryQueryResult, Value};
use rsql_formatters::Results;
use rust_i18n::t;

/// Command to display primary key information.
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("primary_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("primary_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("primary_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let start = std::time::Instant::now();
        let output = options.output;
        let metadata = options.connection.metadata().await?;
        let table_filter = options.input.get(1).map(String::as_str);
        let configuration = options.configuration;
        let locale = &configuration.locale;
        let table_label = t!("table", locale = locale).to_string();
        let pk_label = t!("primary_key_label", locale = locale).to_string();
        let columns_label = t!("primary_columns_label", locale = locale).to_string();
        let inferred_label = t!("primary_inferred_label", locale = locale).to_string();
        let columns = vec![table_label, pk_label, columns_label, inferred_label];
        let mut rows = Vec::new();

        if let Some(catalog) = metadata.current_catalog()
            && let Some(schema) = catalog.current_schema()
        {
            let tables = match table_filter {
                Some(table_name) => match schema.get(table_name) {
                    Some(table) => vec![table],
                    None => Vec::new(),
                },
                None => schema.tables(),
            };

            let list_delimiter = t!("list_delimiter", locale = locale);
            for table in tables {
                if let Some(pk) = table.primary_key() {
                    let inferred = if pk.inferred() {
                        t!("yes", locale = locale).to_string()
                    } else {
                        t!("no", locale = locale).to_string()
                    };
                    let row = vec![
                        Value::String(table.name().to_string()),
                        Value::String(pk.name().to_string()),
                        Value::String(pk.columns().join(&*list_delimiter)),
                        Value::String(inferred),
                    ];
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
    use rsql_drivers::{Metadata, MockConnection, PrimaryKey, Schema, Table};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "primary");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[table]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Display the primary keys");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut metadata = Metadata::new();
        let mut catalog = Catalog::new("default", true);
        let mut schema = Schema::new("default", true);
        let mut table = Table::new("table1");
        table.set_primary_key(PrimaryKey::new("table1_pkey", vec!["id"], false));
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
            input: vec![".primary".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let tables = output.to_string();
        assert!(tables.contains("table1_pkey"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_with_table() -> anyhow::Result<()> {
        let mut metadata = Metadata::new();
        let mut catalog = Catalog::new("default", true);
        let mut schema = Schema::new("default", true);
        let table_name = "table1";
        let mut table = Table::new(table_name);
        table.set_primary_key(PrimaryKey::new("table1_pkey", vec!["id"], false));
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
            input: vec![".primary".to_string(), table_name.to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let tables = output.to_string();
        assert!(tables.contains("table1_pkey"));
        Ok(())
    }
}
