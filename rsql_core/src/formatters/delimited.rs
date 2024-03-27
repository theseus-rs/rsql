use crate::drivers::Results;
use crate::drivers::Results::Query;
use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
use csv::QuoteStyle;

pub async fn format<'a>(
    options: &mut FormatterOptions<'a>,
    delimiter: u8,
    quote_style: QuoteStyle,
    results: &Results,
) -> Result<()> {
    if let Query(query_result) = &results {
        let output = &mut options.output;
        let configuration = &options.configuration;
        let mut writer = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .quote_style(quote_style)
            .from_writer(output);

        if configuration.results_header {
            writer.write_record(query_result.columns().await)?;
        }

        for row in &query_result.rows().await {
            let mut csv_row: Vec<Vec<u8>> = Vec::new();

            for data in row {
                let bytes = if let Some(value) = data {
                    Vec::from(value.to_string().as_bytes())
                } else {
                    Vec::new()
                };
                csv_row.push(bytes);
            }
            writer.write_record(csv_row)?;
        }
        writer.flush()?;
    }

    write_footer(options, results).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::MemoryQueryResult;
    use crate::drivers::Results::{Execute, Query};
    use crate::drivers::Value;
    use crate::formatters::formatter::FormatterOptions;
    use indoc::indoc;
    use std::io::Cursor;
    use std::time::Duration;

    #[tokio::test]
    async fn test_format_execute() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: false,
            ..Default::default()
        };
        let output = &mut Cursor::new(Vec::new());
        let mut options = FormatterOptions {
            configuration,
            elapsed: Duration::from_nanos(9),
            output,
        };

        format(&mut options, b',', QuoteStyle::NonNumeric, &Execute(1))
            .await
            .unwrap();

        let output = String::from_utf8(output.get_ref().to_vec())?.replace("\r\n", "\n");
        let expected = "1 row (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_format_query_no_header_no_footer() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: false,
            results_header: false,
            results_footer: false,
            ..Default::default()
        };
        let query_result = Query(Box::new(MemoryQueryResult::new(
            vec!["id".to_string(), "data".to_string()],
            vec![vec![
                Some(Value::I64(1)),
                Some(Value::String("foo".to_string())),
            ]],
        )));
        let output = &mut Cursor::new(Vec::new());
        let mut options = FormatterOptions {
            configuration,
            elapsed: Duration::from_nanos(9),
            output,
        };

        format(&mut options, b',', QuoteStyle::NonNumeric, &query_result)
            .await
            .unwrap();

        let output = String::from_utf8(output.get_ref().to_vec())?.replace("\r\n", "\n");
        let expected = indoc! {r#"
            1,"foo"
        "#};
        assert_eq!(output, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_format_query() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: false,
            ..Default::default()
        };
        let query_result = Query(Box::new(MemoryQueryResult::new(
            vec!["id".to_string(), "data".to_string()],
            vec![
                vec![Some(Value::I64(1)), Some(Value::Bytes(b"bytes".to_vec()))],
                vec![Some(Value::I64(2)), Some(Value::String("foo".to_string()))],
                vec![Some(Value::I64(3)), None],
            ],
        )));
        let output = &mut Cursor::new(Vec::new());
        let mut options = FormatterOptions {
            configuration,
            elapsed: Duration::from_nanos(9),
            output,
        };

        format(&mut options, b',', QuoteStyle::NonNumeric, &query_result)
            .await
            .unwrap();

        let output = String::from_utf8(output.get_ref().to_vec())?.replace("\r\n", "\n");
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
