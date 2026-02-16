use crate::Results;
use crate::Results::Query;
use crate::error::Result;
use crate::footer::write_footer;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use csv::QuoteStyle;
use rsql_drivers::Value;

pub async fn format(
    options: &FormatterOptions,
    delimiter: u8,
    quote_style: QuoteStyle,
    results: &mut Results,
    output: &mut Output,
) -> Result<()> {
    let rows = format_delimited(options, delimiter, quote_style, results, output).await?;
    write_footer(options, results, rows, output).await
}

async fn format_delimited(
    options: &FormatterOptions,
    delimiter: u8,
    quote_style: QuoteStyle,
    results: &mut Results,
    output: &mut Output,
) -> Result<u64> {
    let mut rows: u64 = 0;

    if let Query(query_result) = results {
        let mut writer = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .quote_style(quote_style)
            .from_writer(output);

        if options.header {
            writer.write_record(query_result.columns())?;
        }

        while let Some(row) = query_result.next().await {
            let mut csv_row: Vec<Vec<u8>> = Vec::new();

            for data in row {
                let bytes = match data {
                    Value::Null => Vec::new(),
                    _ => Vec::from(data.to_string().as_bytes()),
                };
                csv_row.push(bytes);
            }
            writer.write_record(csv_row)?;
            rows += 1;
        }
        writer.flush()?;
    }

    Ok(rows)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Results::Execute;
    use crate::formatter::FormatterOptions;
    use crate::writers::Output;
    use indoc::indoc;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;

    #[tokio::test]
    async fn test_format_execute() -> anyhow::Result<()> {
        let options = FormatterOptions {
            color: false,
            elapsed: Duration::from_nanos(9),
            ..Default::default()
        };
        let output = &mut Output::default();

        format(
            &options,
            b',',
            QuoteStyle::NonNumeric,
            &mut Execute(1),
            output,
        )
        .await
        .unwrap();

        let output = output.to_string().replace("\r\n", "\n");
        let expected = "1 row (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_format_query_no_header_no_footer() -> anyhow::Result<()> {
        let options = FormatterOptions {
            color: false,
            elapsed: Duration::from_nanos(9),
            footer: false,
            header: false,
            ..Default::default()
        };
        let mut query_result = Query(Box::new(MemoryQueryResult::new(
            vec!["id".to_string(), "data".to_string()],
            vec![vec![Value::I64(1), Value::String("foo".to_string())]],
        )));
        let output = &mut Output::default();

        format(
            &options,
            b',',
            QuoteStyle::NonNumeric,
            &mut query_result,
            output,
        )
        .await
        .unwrap();

        let output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r#"
            1,"foo"
        "#};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_format_query() -> anyhow::Result<()> {
        let options = FormatterOptions {
            color: false,
            elapsed: Duration::from_nanos(9),
            ..Default::default()
        };
        let mut query_result = Query(Box::new(MemoryQueryResult::new(
            vec!["id".to_string(), "data".to_string()],
            vec![
                vec![Value::I64(1), Value::Bytes(b"bytes".to_vec())],
                vec![Value::I64(2), Value::String("foo".to_string())],
                vec![Value::I64(3), Value::Null],
            ],
        )));
        let output = &mut Output::default();

        format(
            &options,
            b',',
            QuoteStyle::NonNumeric,
            &mut query_result,
            output,
        )
        .await
        .unwrap();

        let output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r#"
            "id","data"
            1,"Ynl0ZXM="
            2,"foo"
            3,""
            3 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
