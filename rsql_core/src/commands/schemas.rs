use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_drivers::{MemoryQueryResult, Row, Value};
use rsql_formatters::Results;
use rust_i18n::t;

/// List the schemas in the database
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("schemas_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("schemas_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let start = std::time::Instant::now();
        let output = options.output;
        let metadata = options.connection.metadata().await?;
        let configuration = options.configuration;
        let locale = &configuration.locale;
        let schema_label = t!("schema", locale = locale).to_string();
        let current_label = t!("schemas_current", locale = locale).to_string();
        let columns = vec![schema_label, current_label];
        let mut rows = Vec::new();

        let schemas = metadata.schemas();
        for schema in schemas {
            let name = Value::String(schema.name().to_string());
            let current = if schema.current() {
                Value::String(t!("yes", locale = locale).to_string())
            } else {
                Value::String(t!("no", locale = locale).to_string())
            };
            let row = Row::new(vec![name, current]);
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
    use crate::configuration::Configuration;
    use crate::writers::Output;
    use rsql_drivers::{DriverManager, Metadata, MockConnection, Schema};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "schemas");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "List the schemas in the database");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut metadata = Metadata::new();
        let schema_name = "default";
        let schema = Schema::new(schema_name, true);
        metadata.add(schema);

        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_metadata()
            .returning(move || Ok(metadata.clone()));
        let mut output = Output::default();
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".schemas".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let schemas = output.to_string();
        assert!(schemas.contains(schema_name));
        Ok(())
    }
}
