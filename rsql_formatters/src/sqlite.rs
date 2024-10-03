use crate::delimited::format;
use crate::error::Result;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use crate::Results;
use async_trait::async_trait;
use csv::QuoteStyle;

/// A formatter for sqlite tables
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "sqlite"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &mut Results,
        output: &mut Output,
    ) -> Result<()> {
        format(options, b'|', QuoteStyle::Never, results, output).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::formatter::FormatterOptions;
    use crate::writers::Output;
    use crate::Formatter;
    use crate::Results::Query;
    use indoc::indoc;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
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

        let formatter = Formatter;
        formatter
            .format(&options, &mut query_result, output)
            .await?;

        let output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {"
            id|data
            1|Ynl0ZXM=
            2|foo
            3|
            3 rows (9ns)
        "};
        assert_eq!(output, expected);
        Ok(())
    }
}
