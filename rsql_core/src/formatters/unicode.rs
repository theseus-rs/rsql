use crate::formatters::formatter::FormatterOptions;
use crate::formatters::table;
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use prettytable::format::{FormatBuilder, LinePosition, LineSeparator, TableFormat};

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

pub(crate) struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "unicode"
    }

    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()> {
        table::format(*FORMAT_UNICODE, options).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::{QueryResult, Results, Value};
    use crate::formatters::Formatter;
    use rustyline::ColorMode;
    use std::time::Duration;

    fn query_result() -> Results {
        let query_result = QueryResult {
            columns: vec!["id".to_string()],
            rows: vec![vec![Some(Value::I64(12345))]],
        };

        Results::Query(query_result)
    }

    #[tokio::test]
    async fn test_format() -> Result<()> {
        let mut configuration = Configuration {
            color_mode: ColorMode::Disabled,
            ..Default::default()
        };
        let results = query_result();
        let elapsed = Duration::from_nanos(5678);
        let output = &mut Vec::new();
        let mut options = FormatterOptions {
            configuration: &mut configuration,
            results: &results,
            elapsed: &elapsed,
            output,
        };
        let formatter = Formatter;

        formatter.format(&mut options).await?;

        let unicode_output = String::from_utf8(output.clone())?.replace("\r\n", "\n");
        let expected_output =
            "┌────────┐\n│ id     │\n╞════════╡\n│ 12,345 │\n└────────┘\n1 row (5.678µs)\n";
        assert_eq!(unicode_output, expected_output);
        Ok(())
    }
}
