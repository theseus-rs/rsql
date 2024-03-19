use crate::formatters::error::Result;
use crate::formatters::formatter::FormatterOptions;
use crate::formatters::table;
use async_trait::async_trait;
use prettytable::format::consts::FORMAT_DEFAULT;

/// A formatter for ASCII tables
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "ascii"
    }

    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()> {
        table::format(*FORMAT_DEFAULT, options).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::{MemoryQueryResult, Results, Value};
    use crate::formatters::Formatter;
    use indoc::indoc;
    use rustyline::ColorMode;
    use std::time::Duration;

    fn query_result() -> Results {
        let query_result =
            MemoryQueryResult::new(vec!["id".to_string()], vec![vec![Some(Value::I64(12345))]]);

        Results::Query(Box::new(query_result))
    }

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
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

        let ascii_output = String::from_utf8(output.clone())?.replace("\r\n", "\n");
        let expected = indoc! {r#"
            +--------+
            | id     |
            +========+
            | 12,345 |
            +--------+
            1 row (5.678µs)
        "#};
        assert_eq!(ascii_output, expected);
        Ok(())
    }
}
