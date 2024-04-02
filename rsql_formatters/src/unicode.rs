use crate::error::Result;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use crate::{table, Results};
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

/// A formatter for Unicode tables
#[derive(Debug, Default)]
pub(crate) struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "unicode"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &Results,
        output: &mut Output,
    ) -> Result<()> {
        table::format(*FORMAT_UNICODE, options, results, output).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writers::Output;
    use crate::Formatter;
    use indoc::indoc;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;

    fn query_result() -> Results {
        let query_result =
            MemoryQueryResult::new(vec!["id".to_string()], vec![vec![Some(Value::I64(12345))]]);

        Results::Query(Box::new(query_result))
    }

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
        let options = FormatterOptions {
            color: false,
            elapsed: Duration::from_nanos(5678),
            ..Default::default()
        };
        let results = query_result();
        let output = &mut Output::default();
        let formatter = Formatter;

        formatter.format(&options, &results, output).await?;

        let unicode_output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r#"
            ┌────────┐
            │ id     │
            ╞════════╡
            │ 12,345 │
            └────────┘
            1 row (5.678µs)
        "#};
        assert_eq!(unicode_output, expected);
        Ok(())
    }
}
