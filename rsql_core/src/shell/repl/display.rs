use crate::configuration::{Configuration, ResultFormat};
use crate::engine::QueryResult;
use crate::shell::repl::r#impl::SqlResult;
use anyhow::Result;
use colored::Colorize;
use lazy_static::lazy_static;
use num_format::ToFormattedString;
use prettytable::format::consts::FORMAT_DEFAULT;
use prettytable::format::{FormatBuilder, LinePosition, LineSeparator, TableFormat};
use prettytable::Table;
use rustyline::ColorMode;
use std::time::Duration;

lazy_static! {
    pub static ref FORMAT_UNICODE: TableFormat = FormatBuilder::new()
        .column_separator('│')
        .borders('│')
        .separators(&[LinePosition::Top], LineSeparator::new('─', '┬', '┌', '┐'))
        .separators(
            &[LinePosition::Title],
            LineSeparator::new('═', '╪', '╞', '╡')
        )
        .separators(
            &[LinePosition::Intern],
            LineSeparator::new('─', '┼', '├', '┤')
        )
        .separators(
            &[LinePosition::Bottom],
            LineSeparator::new('─', '┴', '└', '┘')
        )
        .padding(1, 1)
        .build();
}

pub(crate) fn table(
    configuration: &Configuration,
    sql_result: SqlResult,
    elapsed: Duration,
) -> Result<()> {
    let rows_affected = match sql_result {
        SqlResult::Execute(rows_affected) => rows_affected,
        SqlResult::Query(query_result) => {
            let mut table = Table::new();

            match configuration.results_format {
                ResultFormat::Ascii => {
                    table.set_format(*FORMAT_DEFAULT);
                }
                ResultFormat::Unicode => {
                    table.set_format(*FORMAT_UNICODE);
                }
            }

            if configuration.results_header {
                process_headers(configuration, &query_result, &mut table);
            }

            process_data(configuration, &query_result, &mut table)?;

            table.printstd();
            query_result.rows.len() as u64
        }
    };

    if configuration.results_footer {
        display_footer(configuration, rows_affected, elapsed)?;
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
    configuration: &Configuration,
    rows_affected: u64,
    elapsed: Duration,
) -> Result<()> {
    let row_label = if rows_affected == 1 { "row" } else { "rows" };
    let elapsed_display = if configuration.results_timer {
        format!("({:?})", elapsed)
    } else {
        "".to_string()
    };

    match configuration.color_mode {
        ColorMode::Disabled => println!(
            "{} {} {}",
            rows_affected.to_formatted_string(&configuration.locale),
            row_label,
            elapsed_display
        ),
        _ => println!(
            "{} {} {}",
            rows_affected.to_formatted_string(&configuration.locale),
            row_label,
            elapsed_display.dimmed()
        ),
    }

    Ok(())
}
