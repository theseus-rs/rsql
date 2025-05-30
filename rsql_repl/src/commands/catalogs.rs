use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_drivers::{MemoryQueryResult, Value};
use rsql_formatters::Results;
use rust_i18n::t;

/// List the catalogs in the database
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("catalogs_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("catalogs_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let start = std::time::Instant::now();
        let output = options.output;
        let metadata = options.connection.metadata().await?;
        let configuration = options.configuration;
        let locale = &configuration.locale;
        let schema_label = t!("catalog", locale = locale).to_string();
        let current_label = t!("catalogs_current", locale = locale).to_string();
        let columns = vec![schema_label, current_label];
        let mut rows = Vec::new();

        let catalogs = metadata.catalogs();
        for schema in catalogs {
            let name = Value::String(schema.name().to_string());
            let current = if schema.current() {
                Value::String(t!("yes", locale = locale).to_string())
            } else {
                Value::String(t!("no", locale = locale).to_string())
            };
            let row = vec![name, current];
            rows.push(row);
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
    use rsql_drivers::{Metadata, MockConnection};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "catalogs");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "List the catalogs in the database");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut metadata = Metadata::new();
        let catalog_name = "default";
        let catalog = Catalog::new(catalog_name, true);
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
            input: vec![".catalogs".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let catalogs = output.to_string();
        assert!(catalogs.contains(catalog_name));
        Ok(())
    }
}
