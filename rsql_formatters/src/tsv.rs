use crate::delimited::format;
use crate::error::Result;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use async_trait::async_trait;
use csv::QuoteStyle;
use rsql_drivers::Results;

/// A formatter for Tab Separated Values (TSV)
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "tsv"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &Results,
        output: &mut Output,
    ) -> Result<()> {
        format(options, b'\t', QuoteStyle::NonNumeric, results, output).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::formatter::FormatterOptions;
    use crate::writers::Output;
    use crate::Formatter;
    use indoc::indoc;
    use rsql_drivers::MemoryQueryResult;
    use rsql_drivers::Results::Query;
    use rsql_drivers::Value;
    use std::time::Duration;

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
        let options = FormatterOptions {
            color: false,
            elapsed: Duration::from_nanos(9),
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
        let output = &mut Output::default();

        let formatter = Formatter;
        formatter.format(&options, &query_result, output).await?;

        let output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {"
            \"id\"\t\"data\"
            1\t\"Ynl0ZXM=\"
            2\t\"foo\"
            3\t\"\"
            3 rows (9ns)
        "};
        assert_eq!(output, expected);
        Ok(())
    }
}
