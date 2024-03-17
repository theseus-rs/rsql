use crate::drivers::QueryResult;
use crate::drivers::Results::Query;
use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
use async_trait::async_trait;
use std::io;

/// A formatter for Column Separated Values (CSV)
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "csv"
    }

    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()> {
        if let Query(query_result) = &options.results {
            self.write_query_results(options, query_result).await?;
        }

        write_footer(options)?;
        Ok(())
    }
}

impl Formatter {
    async fn write_query_results(
        &self,
        options: &mut FormatterOptions<'_>,
        query_result: &QueryResult,
    ) -> Result<()> {
        let configuration = &options.configuration;
        let output = &mut options.output as &mut dyn io::Write;
        let mut writer = csv::WriterBuilder::new()
            .delimiter(b',')
            .quote_style(csv::QuoteStyle::NonNumeric)
            .from_writer(output);

        if configuration.results_header {
            writer.write_record(&query_result.columns)?;
        }
        for row in &query_result.rows {
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
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::QueryResult;
    use crate::drivers::Results::Query;
    use crate::drivers::Value;
    use crate::formatters::formatter::FormatterOptions;
    use crate::formatters::Formatter;
    use rustyline::ColorMode;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color_mode: ColorMode::Disabled,
            ..Default::default()
        };
        let query_result = Query(QueryResult {
            columns: vec!["id".to_string(), "data".to_string()],
            rows: vec![
                vec![Some(Value::I64(1)), Some(Value::Bytes(b"bytes".to_vec()))],
                vec![Some(Value::I64(2)), Some(Value::String("foo".to_string()))],
                vec![Some(Value::I64(3)), None],
            ],
        });
        let output = &mut Cursor::new(Vec::new());
        let mut options = FormatterOptions {
            configuration,
            results: &query_result,
            elapsed: &std::time::Duration::from_nanos(9),
            output,
        };

        let formatter = Formatter;
        formatter.format(&mut options).await.unwrap();

        let output = String::from_utf8(output.get_ref().to_vec())?.replace("\r\n", "\n");
        let expected = "\"id\",\"data\"\n1,\"Ynl0ZXM=\"\n2,\"foo\"\n3,\"\"\n3 rows (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }
}
