use crate::drivers::Results;
use crate::formatters::error::Result;
use crate::formatters::formatter::FormatterOptions;
use crate::formatters::table;
use async_trait::async_trait;
use lazy_static::lazy_static;
use prettytable::format::{FormatBuilder, LinePosition, LineSeparator, TableFormat};

lazy_static! {
    pub static ref FORMAT_MARKDOWN: TableFormat = FormatBuilder::new()
        .column_separator('|')
        .borders('|')
        .separators(
            &[LinePosition::Title],
            LineSeparator::new('-', '|', '|', '|')
        )
        .padding(1, 1)
        .build();
}

/// A formatter for markdown tables
#[derive(Debug, Default)]
pub(crate) struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "markdown"
    }

    async fn format<'a>(
        &self,
        options: &mut FormatterOptions<'a>,
        results: &Results,
    ) -> Result<()> {
        table::format(*FORMAT_MARKDOWN, options, results).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::{MemoryQueryResult, Results, Value};
    use crate::formatters::Formatter;
    use indoc::indoc;
    use std::time::Duration;

    fn query_result() -> Results {
        let query_result =
            MemoryQueryResult::new(vec!["id".to_string()], vec![vec![Some(Value::I64(12345))]]);

        Results::Query(Box::new(query_result))
    }

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            color: false,
            ..Default::default()
        };
        let results = query_result();
        let output = &mut Vec::new();
        let mut options = FormatterOptions {
            configuration: &mut configuration,
            elapsed: Duration::from_nanos(5678),
            output,
        };
        let formatter = Formatter;

        formatter.format(&mut options, &results).await?;

        let plain_output = String::from_utf8(output.clone())?.replace("\r\n", "\n");
        let expected = indoc! {r#"
            | id     |
            |--------|
            | 12,345 |
            1 row (5.678µs)
        "#};
        assert_eq!(plain_output, expected);
        Ok(())
    }
}
