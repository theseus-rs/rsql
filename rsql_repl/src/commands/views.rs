use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_drivers::{MemoryQueryResult, Value};
use rsql_formatters::Results;
use rust_i18n::t;

/// List the views in the schema
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("views_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("views_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let start = std::time::Instant::now();
        let output = options.output;
        let metadata = options.connection.metadata().await?;
        let configuration = options.configuration;
        let locale = &configuration.locale;
        let view_label = t!("view", locale = locale).to_string();
        let columns = vec![view_label];
        let mut rows = Vec::new();

        if let Some(catalog) = metadata.current_catalog()
            && let Some(schema) = catalog.current_schema()
        {
            let views = schema.views();
            for view in views {
                let value = Value::String(view.name().to_string());
                let row = vec![value];
                rows.push(row);
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
    use rsql_drivers::{Column, Metadata, MockConnection, Schema, View};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "views");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "List the views in the schema");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut metadata = Metadata::new();
        let mut catalog = Catalog::new("default", true);
        let mut schema = Schema::new("default", true);
        let view_name = "active_users";
        let mut view = View::new(view_name);
        view.add_column(Column::new("id", "INTEGER", true, None));
        schema.add_view(view);
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
            input: vec![".views".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let views = output.to_string();
        assert!(views.contains(view_name));
        Ok(())
    }
}
