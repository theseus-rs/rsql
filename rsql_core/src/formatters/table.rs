use crate::configuration::Configuration;
use crate::drivers::{QueryResult, Results};
use crate::formatters::formatter::FormatterOptions;
use anyhow::Result;
use colored::Colorize;
use num_format::ToFormattedString;
use prettytable::format::TableFormat;
use prettytable::Table;
use rustyline::ColorMode;
use std::io::Write;
use std::time::Duration;

/// Format the results of a query into a table and write to the output.
pub async fn format<'a>(
    table_format: TableFormat,
    options: &mut FormatterOptions<'a>,
) -> Result<()> {
    let configuration = &options.configuration;
    let output = &mut options.output;
    let rows_affected = match options.results {
        Results::Execute(rows_affected) => *rows_affected,
        Results::Query(query_result) => {
            let mut table = Table::new();
            table.set_format(table_format);

            if configuration.results_header {
                process_headers(configuration, query_result, &mut table);
            }

            process_data(configuration, query_result, &mut table)?;

            table.print(output)?;
            query_result.rows.len() as u64
        }
    };

    if configuration.results_footer {
        let elapsed = options.elapsed;
        display_footer(output, configuration, rows_affected, elapsed)?;
    }

    Ok(())
}

fn process_headers(configuration: &Configuration, query_result: &QueryResult, table: &mut Table) {
    let mut column_names = Vec::new();

    for column in &query_result.columns {
        match configuration.color_mode {
            ColorMode::Disabled => {
                column_names.push(column.to_string());
            }
            _ => {
                column_names.push(column.green().bold().to_string());
            }
        }
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

/// Display the footer of the result set.
/// This includes the number of rows returned and the elapsed time.
/// If the timing option is enabled, the elapsed time will be displayed.
/// The number of rows will be formatted based on the locale.
///
/// Example: "N,NNN,NNN rows (M.MMMs)"
pub(crate) fn display_footer(
    output: &mut dyn Write,
    configuration: &Configuration,
    rows_affected: u64,
    elapsed: &Duration,
) -> Result<()> {
    let row_label = if rows_affected == 1 { "row" } else { "rows" };
    let elapsed_display = if configuration.results_timer {
        format!("({:?})", elapsed)
    } else {
        "".to_string()
    };

    match configuration.color_mode {
        ColorMode::Disabled => writeln!(
            output,
            "{} {} {}",
            rows_affected.to_formatted_string(&configuration.locale),
            row_label,
            elapsed_display
        )?,
        _ => writeln!(
            output,
            "{} {} {}",
            rows_affected.to_formatted_string(&configuration.locale),
            row_label,
            elapsed_display.dimmed()
        )?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::{QueryResult, Results, Value};
    use num_format::Locale;
    use prettytable::format::consts::FORMAT_DEFAULT;
    use std::time::Duration;

    const COLUMN_HEADER: &str = "column 1";

    fn query_result_no_rows() -> Results {
        let query_result = QueryResult {
            columns: vec![COLUMN_HEADER.to_string()],
            rows: vec![],
        };

        Results::Query(query_result)
    }

    fn query_result_one_row() -> Results {
        let query_result = QueryResult {
            columns: vec![COLUMN_HEADER.to_string()],
            rows: vec![vec![Some(Value::I64(12345))]],
        };

        Results::Query(query_result)
    }

    async fn test_format(configuration: &mut Configuration, results: &Results) -> Result<Vec<u8>> {
        let elapsed = Duration::from_nanos(9);
        let output = &mut Vec::new();
        let mut options = FormatterOptions {
            configuration,
            results: &results,
            elapsed: &elapsed,
            output,
        };

        format(*FORMAT_DEFAULT, &mut options).await?;

        let ascii_output = String::from_utf8(output.clone())?;
        match results {
            Results::Execute(_) => {
                assert!(!ascii_output.contains("+-"));
                assert!(!ascii_output.contains("-+"));
                assert!(!ascii_output.contains(COLUMN_HEADER));
                assert!(!ascii_output.contains("+="));
                assert!(!ascii_output.contains("=+"));
            }
            Results::Query(_) => {
                assert!(ascii_output.contains("+-"));
                assert!(ascii_output.contains("-+"));

                if configuration.results_header {
                    assert!(ascii_output.contains("+="));
                    assert!(ascii_output.contains("=+"));
                    assert!(ascii_output.contains(COLUMN_HEADER));
                }
            }
        }

        if configuration.results_footer && configuration.results_timer {
            assert!(ascii_output.contains("(9ns)"));
        }

        Ok(output.clone())
    }

    #[tokio::test]
    async fn test_execute_format() -> Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            ..Default::default()
        };
        let results = Results::Execute(42);

        let output = test_format(&mut configuration, &results).await?;
        let ascii_output = String::from_utf8(output.clone())?;
        assert!(ascii_output.contains("42 rows"));
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_no_rows() -> Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            ..Default::default()
        };
        let results = query_result_no_rows();

        let output = test_format(&mut configuration, &results).await?;
        let ascii_output = String::from_utf8(output.clone())?;
        assert!(ascii_output.contains("0 rows"));
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_one_row() -> Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            ..Default::default()
        };
        let results = query_result_one_row();

        let output = test_format(&mut configuration, &results).await?;
        let ascii_output = String::from_utf8(output.clone())?;
        assert!(ascii_output.contains("12,345"));
        assert!(ascii_output.contains("1 row"));
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_no_header_and_no_footer() -> Result<()> {
        let mut configuration = Configuration {
            locale: Locale::en,
            results_header: false,
            results_footer: false,
            ..Default::default()
        };
        let results = query_result_one_row();

        let output = test_format(&mut configuration, &results).await?;
        let ascii_output = String::from_utf8(output.clone())?;
        assert!(ascii_output.contains("12,345"));
        assert!(!ascii_output.contains("1 row"));
        Ok(())
    }
}
