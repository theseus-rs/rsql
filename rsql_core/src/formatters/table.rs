use crate::configuration::Configuration;
use crate::drivers::QueryResult;
use crate::drivers::Results::Query;
use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
use colored::Colorize;
use prettytable::format::TableFormat;
use prettytable::Table;
use rustyline::ColorMode;

/// Format the results of a query into a table and write to the output.
pub async fn format<'a>(
    table_format: TableFormat,
    options: &mut FormatterOptions<'a>,
) -> Result<()> {
    let configuration = &options.configuration;
    let output = &mut options.output;

    if let Query(query_result) = &options.results {
        let mut table = Table::new();
        table.set_format(table_format);

        if configuration.results_header {
            process_headers(query_result, &mut table);
        }

        process_data(configuration, query_result, &mut table)?;

        table.print(output)?;
    }

    write_footer(options)?;
    Ok(())
}

fn process_headers(query_result: &QueryResult, table: &mut Table) {
    let mut column_names = Vec::new();

    for column in &query_result.columns {
        column_names.push(column.to_string());
    }

    table.set_titles(prettytable::Row::from(column_names));
}

fn process_data(
    configuration: &Configuration,
    query_result: &QueryResult,
    table: &mut Table,
) -> Result<()> {
    for (i, row) in query_result.rows.iter().enumerate() {
        let mut row_data = Vec::new();

        for data in row {
            let data = match data {
                Some(data) => data.to_formatted_string(&configuration.locale),
                None => "NULL".to_string(),
            };

            match configuration.color_mode {
                ColorMode::Disabled => {
                    row_data.push(data);
                }
                _ => {
                    if i % 2 == 0 {
                        row_data.push(data.dimmed().to_string());
                    } else {
                        row_data.push(data);
                    }
                }
            }
        }

        table.add_row(prettytable::Row::from(row_data));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::Results::{Execute, Query};
    use crate::drivers::{QueryResult, Results, Value};
    use indoc::indoc;
    use num_format::Locale;
    use prettytable::format::consts::FORMAT_DEFAULT;
    use std::time::Duration;

    const COLUMN_HEADER: &str = "id";

    fn query_result_no_rows() -> Results {
        let query_result = QueryResult {
            columns: vec![COLUMN_HEADER.to_string()],
            rows: vec![],
        };

        Query(query_result)
    }

    fn query_result_one_row() -> Results {
        let query_result = QueryResult {
            columns: vec![COLUMN_HEADER.to_string()],
            rows: vec![vec![Some(Value::I64(12345))]],
        };

        Query(query_result)
    }

    fn query_result_two_rows() -> Results {
        let query_result = QueryResult {
            columns: vec![COLUMN_HEADER.to_string()],
            rows: vec![vec![None], vec![Some(Value::I64(12345))]],
        };

        Query(query_result)
    }

    async fn test_format(
        configuration: &mut Configuration,
        results: &Results,
    ) -> anyhow::Result<String> {
        let elapsed = Duration::from_nanos(9);
        let output = &mut Vec::new();
        let mut options = FormatterOptions {
            configuration,
            results: &results,
            elapsed: &elapsed,
            output,
        };

        format(*FORMAT_DEFAULT, &mut options).await?;

        Ok(String::from_utf8(output.clone())?.replace("\r\n", "\n"))
    }

    #[tokio::test]
    async fn test_execute_format() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            color_mode: ColorMode::Disabled,
            ..Default::default()
        };
        let results = Execute(42);

        let output = test_format(&mut configuration, &results).await?;
        let expected = "42 rows (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_no_rows() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            color_mode: ColorMode::Disabled,
            ..Default::default()
        };
        let results = query_result_no_rows();

        let output = test_format(&mut configuration, &results).await?;
        let expected = indoc! {r#"
            +----+
            | id |
            +====+
            +----+
            0 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_footer_no_timer() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            color_mode: ColorMode::Disabled,
            results_footer: true,
            results_timer: false,
            ..Default::default()
        };
        let results = query_result_no_rows();

        let output = test_format(&mut configuration, &results).await?;
        let expected = indoc! {r#"
            +----+
            | id |
            +====+
            +----+
            0 rows 
        "#};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_two_rows_without_color() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            color_mode: ColorMode::Disabled,
            ..Default::default()
        };
        let results = query_result_two_rows();

        let output = test_format(&mut configuration, &results).await?;
        let expected = indoc! {r#"
            +--------+
            | id     |
            +========+
            | NULL   |
            +--------+
            | 12,345 |
            +--------+
            2 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_two_rows_with_color() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            color_mode: ColorMode::Forced,
            ..Default::default()
        };
        let results = query_result_two_rows();

        let output = test_format(&mut configuration, &results).await?;
        assert!(output.contains("id"));
        assert!(output.contains("NULL"));
        assert!(output.contains("12,345"));
        assert!(output.contains("2 rows"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_no_header_and_no_footer() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            color_mode: ColorMode::Disabled,
            results_header: false,
            results_footer: false,
            ..Default::default()
        };
        let results = query_result_one_row();

        let output = test_format(&mut configuration, &results).await?;
        let expected = indoc! {r#"
            +--------+
            | 12,345 |
            +--------+
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
