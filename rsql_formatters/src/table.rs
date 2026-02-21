use crate::Results;
use crate::Results::Query;
use crate::error::Result;
use crate::footer::write_footer;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use rsql_drivers::{QueryResult, Value, ValueFormatter};
use tabled::builder::Builder;
use tabled::settings::object::{Cell, Rows};
use tabled::settings::{Alignment, Theme};

/// Format the results of a query into a table and write to the output.
pub async fn format(
    theme: Theme,
    options: &FormatterOptions,
    results: &mut Results,
    output: &mut Output,
) -> Result<()> {
    let mut rows: u64 = 0;

    if let Query(query_result) = results {
        if query_result.columns().is_empty() {
            write_footer(options, results, 0, output).await?;
            return Ok(());
        }

        let mut builder = Builder::default();

        if options.header {
            builder.push_record(query_result.columns());
        }

        let cells;
        (rows, cells) = process_data(options, query_result, &mut builder).await?;

        let mut table = builder.build();
        table.with(theme);

        if options.header {
            table.modify(Rows::first(), Alignment::center());
        }

        // Align numeric columns to the right
        for cell in cells {
            table.modify(cell, Alignment::right());
        }

        writeln!(output, "{table}")?;
    }

    write_footer(options, results, rows, output).await?;
    Ok(())
}

async fn process_data(
    options: &FormatterOptions,
    query_result: &mut Box<dyn QueryResult>,
    builder: &mut Builder,
) -> Result<(u64, Vec<Cell>)> {
    let mut rows: u64 = 0;
    let mut cells = Vec::new();
    let mut raw_rows: Vec<Vec<Value>> = Vec::new();
    while let Some(row) = query_result.next().await {
        raw_rows.push(row.clone());
    }

    let value_formatter = ValueFormatter::new(options.locale.as_str());
    for row in &raw_rows {
        let mut row_data = Vec::new();

        for (column, data) in row.iter().enumerate() {
            let data = if *data == Value::Null {
                "NULL".to_string()
            } else {
                if data.is_numeric() {
                    let row = if options.header { rows + 1 } else { rows };
                    let cell = Cell::new(usize::try_from(row)?, column);
                    cells.push(cell);
                }
                data.to_formatted_string(&value_formatter)
            };

            row_data.push(data);
        }

        rows += 1;
        builder.push_record(row_data);
    }

    Ok((rows, cells))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Results::Execute;
    use crate::writers::Output;
    use indoc::indoc;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;
    use tabled::settings::Style;

    const COLUMN_HEADER: &str = "id";

    fn query_result_no_columns() -> Results {
        let query_result = MemoryQueryResult::new(vec![], vec![]);
        Query(Box::new(query_result))
    }

    fn query_result_no_rows() -> Results {
        let query_result = MemoryQueryResult::new(vec![COLUMN_HEADER.to_string()], vec![]);
        Query(Box::new(query_result))
    }

    fn query_result_one_row() -> Results {
        let query_result = MemoryQueryResult::new(
            vec![COLUMN_HEADER.to_string()],
            vec![vec![Value::I64(12345)]],
        );
        Query(Box::new(query_result))
    }

    fn query_result_two_rows() -> Results {
        let query_result = MemoryQueryResult::new(
            vec![COLUMN_HEADER.to_string()],
            vec![vec![Value::Null], vec![Value::I64(12345)]],
        );
        Query(Box::new(query_result))
    }

    fn query_result_number_and_string() -> Results {
        let query_result = MemoryQueryResult::new(
            vec![
                "number".to_string(),
                "string".to_string(),
                "text".to_string(),
            ],
            vec![vec![
                Value::I64(42),
                Value::String("foo".to_string()),
                Value::String("Lorem ipsum dolor sit amet".to_string()),
            ]],
        );
        Query(Box::new(query_result))
    }

    async fn test_format(
        options: &mut FormatterOptions,
        results: &mut Results,
    ) -> anyhow::Result<String> {
        let theme = Theme::from_style(Style::ascii());
        let output = &mut Output::default();
        options.elapsed = Duration::from_nanos(9);

        format(theme, options, results, output).await?;

        Ok(output.to_string().replace("\r\n", "\n"))
    }

    #[tokio::test]
    async fn test_execute_format() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: false,
            locale: "en".to_string(),
            ..Default::default()
        };
        let mut results = Execute(42);

        let output = test_format(&mut options, &mut results).await?;
        let expected = "42 rows (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_no_rows() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: false,
            locale: "en".to_string(),
            ..Default::default()
        };
        let mut results = query_result_no_rows();

        let output = test_format(&mut options, &mut results).await?;
        let expected = indoc! {r"
            +----+
            | id |
            +----+
            0 rows (9ns)
        "};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_footer_no_timer() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: false,
            footer: true,
            locale: "en".to_string(),
            timer: false,
            ..Default::default()
        };
        let mut results = query_result_no_rows();

        let output = test_format(&mut options, &mut results).await?;
        let expected = indoc! {r"
            +----+
            | id |
            +----+
            0 rows
        "};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_two_rows_without_color() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: false,
            locale: "en".to_string(),
            ..Default::default()
        };
        let mut results = query_result_two_rows();

        let output = test_format(&mut options, &mut results).await?;
        let expected = indoc! {r"
            +--------+
            |   id   |
            +--------+
            | NULL   |
            +--------+
            | 12,345 |
            +--------+
            2 rows (9ns)
        "};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_two_rows_with_color() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: true,
            locale: "en".to_string(),
            ..Default::default()
        };
        let mut results = query_result_two_rows();

        let output = test_format(&mut options, &mut results).await?;
        assert!(output.contains("id"));
        assert!(output.contains("NULL"));
        assert!(output.contains("12,345"));
        assert!(output.contains("2 rows"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_no_header_and_no_footer() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: false,
            footer: false,
            header: false,
            locale: "en".to_string(),
            ..Default::default()
        };
        let mut results = query_result_one_row();

        let output = test_format(&mut options, &mut results).await?;
        let expected = indoc! {r"
            +--------+
            | 12,345 |
            +--------+
        "};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_format_no_columns() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: false,
            locale: "en".to_string(),
            ..Default::default()
        };
        let mut results = query_result_no_columns();

        let output = test_format(&mut options, &mut results).await?;
        let expected = indoc! {r"
            0 rows (9ns)
        "};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_query_align_numbers_and_strings() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: false,
            locale: "en".to_string(),
            ..Default::default()
        };
        let mut results = query_result_number_and_string();

        let output = test_format(&mut options, &mut results).await?;
        let expected = indoc! {r"
            +--------+--------+----------------------------+
            | number | string |            text            |
            +--------+--------+----------------------------+
            |     42 | foo    | Lorem ipsum dolor sit amet |
            +--------+--------+----------------------------+
            1 row (9ns)
        "};
        assert_eq!(output, expected);
        Ok(())
    }
}
