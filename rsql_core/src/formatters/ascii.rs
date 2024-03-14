use crate::formatters::formatter::FormatterOptions;
use crate::formatters::table;
use anyhow::Result;
use async_trait::async_trait;
use prettytable::format::consts::FORMAT_DEFAULT;

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
    use crate::drivers::{QueryResult, Results, Value};
    use crate::formatters::Formatter;
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
        let mut configuration = Configuration::default();
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

        let ascii_output = String::from_utf8(output.clone())?;
        assert!(ascii_output.contains("+--------+"));
        assert!(ascii_output.contains("+========+"));
        assert!(ascii_output.contains("12,345"));
        Ok(())
    }
}
