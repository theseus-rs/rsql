use crate::commands::Error::{InvalidOption, MissingArguments};
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_drivers::{MemoryQueryResult, Row, Table, Value};
use rsql_formatters::Results;
use rust_i18n::t;
use std::ops::Deref;

/// Describe the specified database object
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("describe_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("describe_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("describe_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let configuration = options.configuration;
        let output = options.output;
        let locale = &configuration.locale;

        if options.input.len() <= 1 {
            return Err(MissingArguments {
                command_name: self.name(locale).to_string(),
                arguments: self.args(locale).to_string(),
            });
        }

        let metadata = options.connection.metadata().await?;
        let table_name = &options.input[1];

        let column_label = t!("describe_column", locale = locale).to_string();
        let type_label = t!("describe_type", locale = locale).to_string();
        let nullable_label = t!("describe_not_null", locale = locale).to_string();
        let default_label = t!("describe_default", locale = locale).to_string();
        let table_column_labels = vec![column_label, type_label, nullable_label, default_label];
        let mut table_column_rows = Vec::new();

        let index_label = t!("index", locale = locale).to_string();
        let columns_label = t!("describe_columns", locale = locale).to_string();
        let unique_label = t!("describe_unique", locale = locale).to_string();
        let indexes_column_labels = vec![index_label, columns_label, unique_label];
        let mut indexes_column_rows = Vec::new();
        let mut table: Option<&Table> = None;

        if let Some(database) = metadata.current_database() {
            table = database.get(table_name);
        }

        if let Some(table) = table {
            for column in table.columns() {
                let nullable = if column.not_null() {
                    t!("no", locale = locale).to_string()
                } else {
                    t!("yes", locale = locale).to_string()
                };
                let row = Row::new(vec![
                    Value::String(column.name().to_string()),
                    Value::String(column.data_type().to_string()),
                    Value::String(nullable),
                    Value::String(column.default().unwrap_or("").to_string()),
                ]);
                table_column_rows.push(row);
            }

            let list_delimiter = t!("list_delimiter", locale = locale);
            for index in table.indexes() {
                let unique = if index.unique() {
                    t!("yes", locale = locale).to_string()
                } else {
                    t!("no", locale = locale).to_string()
                };
                let row = Row::new(vec![
                    Value::String(index.name().to_string()),
                    Value::String(index.columns().join(list_delimiter.deref())),
                    Value::String(unique),
                ]);
                indexes_column_rows.push(row);
            }
        } else {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: table_name.to_string(),
            });
        }

        let query_result = MemoryQueryResult::new(table_column_labels, table_column_rows);
        let mut table_results = Results::Query(Box::new(query_result));
        let query_result = MemoryQueryResult::new(indexes_column_labels, indexes_column_rows);
        let mut indexes_results = Results::Query(Box::new(query_result));

        let formatter_options = &mut configuration.get_formatter_options();
        let result_format = &configuration.results_format;
        let formatter = options.formatter_manager.get(result_format).ok_or(
            rsql_formatters::Error::UnknownFormat {
                format: result_format.to_string(),
            },
        )?;

        // Explicitly set the header to true and the footer to false and restore the values before
        // returning from the function
        let header = formatter_options.header;
        let footer = formatter_options.footer;
        formatter_options.header = true;
        formatter_options.footer = false;

        formatter
            .format(formatter_options, &mut table_results, output)
            .await?;

        let indexes_label = t!("describe_indexes", locale = locale).to_string();
        writeln!(output, "{}", indexes_label)?;
        formatter
            .format(formatter_options, &mut indexes_results, output)
            .await?;

        formatter_options.header = header;
        formatter_options.footer = footer;
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
    use indoc::indoc;
    use rsql_drivers::{Column, Database, DriverManager, Index, Metadata, MockConnection, Table};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;
    use std::default;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "describe");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[table]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Describe a table in the database");
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".describe".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_table() -> anyhow::Result<()> {
        let mut metadata = Metadata::default();
        let database = Database::default();
        metadata.add(database);
        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_metadata()
            .returning(move || Ok(metadata.clone()));

        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".describe".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: false,
            ..default::Default::default()
        };
        let mut metadata = Metadata::default();
        let mut database = Database::default();
        let table_name = "users";
        let mut table = Table::new(table_name);
        table.add_column(Column::new("id", "INTEGER", true, None));
        table.add_column(Column::new("name", "TEXT", false, None));
        table.add_index(Index::new("users_id_idx", vec!["id"], true));
        table.add_index(Index::new("users_name_idx", vec!["name"], false));
        database.add(table);
        metadata.add(database);

        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_metadata()
            .returning(move || Ok(metadata.clone()));
        let mut output = Output::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".describe".to_string(), table_name.to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let contents = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r#"
            ┌────────┬─────────┬──────────┬─────────┐
            │ Column │  Type   │ Not null │ Default │
            ╞════════╪═════════╪══════════╪═════════╡
            │ id     │ INTEGER │ No       │         │
            ├────────┼─────────┼──────────┼─────────┤
            │ name   │ TEXT    │ Yes      │         │
            └────────┴─────────┴──────────┴─────────┘
            Indexes
            ┌────────────────┬─────────┬────────┐
            │     Index      │ Columns │ Unique │
            ╞════════════════╪═════════╪════════╡
            │ users_id_idx   │ id      │ Yes    │
            ├────────────────┼─────────┼────────┤
            │ users_name_idx │ name    │ No     │
            └────────────────┴─────────┴────────┘
        "#};
        assert_eq!(contents, expected);

        Ok(())
    }
}
