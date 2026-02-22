use crate::commands::Error::{InvalidOption, MissingArguments};
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_drivers::{MemoryQueryResult, Table, Value, View};
use rsql_formatters::Results;
use rust_i18n::t;

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
        let object_name = &options.input[1];

        let column_label = t!("describe_column", locale = locale).to_string();
        let type_label = t!("describe_type", locale = locale).to_string();
        let nullable_label = t!("describe_not_null", locale = locale).to_string();
        let default_label = t!("describe_default", locale = locale).to_string();
        let table_column_labels = vec![column_label, type_label, nullable_label, default_label];
        let mut table_column_rows = Vec::new();

        let index_label = t!("index", locale = locale).to_string();
        let index_columns_label = t!("describe_columns", locale = locale).to_string();
        let index_unique_label = t!("describe_unique", locale = locale).to_string();
        let indexes_column_labels = vec![index_label, index_columns_label, index_unique_label];
        let mut indexes_column_rows = Vec::new();

        let pk_label = t!("describe_primary_key", locale = locale).to_string();
        let pk_columns_label = t!("describe_columns", locale = locale).to_string();
        let pk_inferred_label = t!("describe_inferred", locale = locale).to_string();
        let pk_column_labels = vec![pk_label, pk_columns_label, pk_inferred_label];
        let mut pk_rows = Vec::new();

        let fk_label = t!("describe_foreign_key", locale = locale).to_string();
        let fk_columns_label = t!("describe_columns", locale = locale).to_string();
        let fk_ref_table_label = t!("describe_referenced_table", locale = locale).to_string();
        let fk_ref_columns_label = t!("describe_referenced_columns", locale = locale).to_string();
        let fk_inferred_label = t!("describe_inferred", locale = locale).to_string();
        let fk_column_labels = vec![
            fk_label,
            fk_columns_label,
            fk_ref_table_label,
            fk_ref_columns_label,
            fk_inferred_label,
        ];
        let mut fk_rows = Vec::new();

        let mut table: Option<&Table> = None;
        let mut view: Option<&View> = None;

        if let Some(catalog) = metadata.current_catalog()
            && let Some(schema) = catalog.current_schema()
        {
            table = schema.get(object_name);
            if table.is_none() {
                view = schema.get_view(object_name);
            }
        }

        let is_view = view.is_some();

        // Collect columns from either a table or a view
        let columns_iter: Vec<&rsql_drivers::Column> = if let Some(table) = table {
            table.columns()
        } else if let Some(view) = view {
            view.columns()
        } else {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: object_name.to_string(),
            });
        };

        for column in columns_iter {
            let nullable = if column.not_null() {
                t!("no", locale = locale).to_string()
            } else {
                t!("yes", locale = locale).to_string()
            };
            let row = vec![
                Value::String(column.name().to_string()),
                Value::String(column.data_type().to_string()),
                Value::String(nullable),
                Value::String(column.default().unwrap_or("").to_string()),
            ];
            table_column_rows.push(row);
        }

        // Only collect indexes, primary keys, and foreign keys for tables (not views)
        if let Some(table) = table {
            let list_delimiter = t!("list_delimiter", locale = locale);
            for index in table.indexes() {
                let unique = if index.unique() {
                    t!("yes", locale = locale).to_string()
                } else {
                    t!("no", locale = locale).to_string()
                };
                let row = vec![
                    Value::String(index.name().to_string()),
                    Value::String(index.columns().join(&*list_delimiter)),
                    Value::String(unique),
                ];
                indexes_column_rows.push(row);
            }

            let list_delimiter_fk = t!("list_delimiter", locale = locale);

            if let Some(pk) = table.primary_key() {
                let list_delimiter_pk = t!("list_delimiter", locale = locale);
                let inferred = if pk.inferred() {
                    t!("yes", locale = locale).to_string()
                } else {
                    t!("no", locale = locale).to_string()
                };
                let row = vec![
                    Value::String(pk.name().to_string()),
                    Value::String(pk.columns().join(&*list_delimiter_pk)),
                    Value::String(inferred),
                ];
                pk_rows.push(row);
            }

            for fk in table.foreign_keys() {
                let inferred = if fk.inferred() {
                    t!("yes", locale = locale).to_string()
                } else {
                    t!("no", locale = locale).to_string()
                };
                let row = vec![
                    Value::String(fk.name().to_string()),
                    Value::String(fk.columns().join(&*list_delimiter_fk)),
                    Value::String(fk.referenced_table().to_string()),
                    Value::String(fk.referenced_columns().join(&*list_delimiter_fk)),
                    Value::String(inferred),
                ];
                fk_rows.push(row);
            }
        }

        let query_result = MemoryQueryResult::new(table_column_labels, table_column_rows);
        let mut table_results = Results::Query(Box::new(query_result));

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

        if !is_view {
            if !indexes_column_rows.is_empty() {
                let query_result =
                    MemoryQueryResult::new(indexes_column_labels, indexes_column_rows);
                let mut indexes_results = Results::Query(Box::new(query_result));
                writeln!(output)?;
                formatter
                    .format(formatter_options, &mut indexes_results, output)
                    .await?;
            }

            if !pk_rows.is_empty() {
                let query_result = MemoryQueryResult::new(pk_column_labels, pk_rows);
                let mut pk_results = Results::Query(Box::new(query_result));
                writeln!(output)?;
                formatter
                    .format(formatter_options, &mut pk_results, output)
                    .await?;
            }

            if !fk_rows.is_empty() {
                let query_result = MemoryQueryResult::new(fk_column_labels, fk_rows);
                let mut fk_results = Results::Query(Box::new(query_result));
                writeln!(output)?;
                formatter
                    .format(formatter_options, &mut fk_results, output)
                    .await?;
            }
        }

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
    use crate::writers::Output;
    use indoc::indoc;
    use rsql_core::Configuration;
    use rsql_driver::Catalog;
    use rsql_drivers::{
        Column, ForeignKey, Index, Metadata, MockConnection, PrimaryKey, Schema, Table, View,
    };
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "describe");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[table|view]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Describe a table or view in the schema");
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
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
        let mut catalog = Catalog::new("default", true);
        let schema = Schema::default();
        catalog.add(schema);
        metadata.add(catalog);
        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_metadata()
            .returning(move || Ok(metadata.clone()));

        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
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
            results_format: "psql".to_string(),
            ..Default::default()
        };
        let mut metadata = Metadata::default();
        let mut catalog = Catalog::new("default", true);
        let mut schema = Schema::new("default", true);
        let table_name = "users";
        let mut table = Table::new(table_name);
        table.add_column(Column::new("id", "INTEGER", true, None));
        table.add_column(Column::new("name", "TEXT", false, None));
        table.add_index(Index::new("users_id_idx", vec!["id"], true));
        table.add_index(Index::new("users_name_idx", vec!["name"], false));
        table.set_primary_key(PrimaryKey::new("users_pkey", vec!["id"], false));
        table.add_foreign_key(ForeignKey::new(
            "fk_users_org",
            vec!["org_id"],
            "organizations",
            vec!["id"],
            false,
        ));
        schema.add(table);
        catalog.add(schema);
        metadata.add(catalog);

        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_metadata()
            .returning(move || Ok(metadata.clone()));
        let mut output = Output::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".describe".to_string(), table_name.to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let contents = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r"
              Column |  Type   | Not null | Default 
             --------+---------+----------+---------
              id     | INTEGER | No       |         
              name   | TEXT    | Yes      |         
             
                  Index      | Columns | Unique
             ----------------+---------+--------
              users_id_idx   | id      | Yes    
              users_name_idx | name    | No     
             
              Primary Key | Columns | Inferred
             -------------+---------+----------
              users_pkey  | id      | No       
             
              Foreign Key  | Columns | Referenced Table | Referenced Columns | Inferred
             --------------+---------+------------------+--------------------+----------
              fk_users_org | org_id  | organizations    | id                 | No       
        "};
        let normalize = |s: &str| -> String {
            s.lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(normalize(&contents), normalize(expected));

        Ok(())
    }

    #[cfg(feature = "format-psql")]
    #[tokio::test]
    async fn test_execute_view() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: false,
            results_format: "psql".to_string(),
            ..Default::default()
        };
        let mut metadata = Metadata::default();
        let mut catalog = Catalog::new("default", true);
        let mut schema = Schema::new("default", true);
        let view_name = "active_users";
        let mut view = View::new(view_name);
        view.add_column(Column::new("id", "INTEGER", true, None));
        view.add_column(Column::new("name", "TEXT", false, None));
        schema.add_view(view);
        catalog.add(schema);
        metadata.add(catalog);

        let mock_connection = &mut MockConnection::new();
        mock_connection
            .expect_metadata()
            .returning(move || Ok(metadata.clone()));
        let mut output = Output::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: mock_connection,
            history: &DefaultHistory::new(),
            input: vec![".describe".to_string(), view_name.to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let contents = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r"
              Column |  Type   | Not null | Default 
             --------+---------+----------+---------
              id     | INTEGER | No       |         
              name   | TEXT    | Yes      |         
        "};
        assert_eq!(contents, expected);

        Ok(())
    }
}
