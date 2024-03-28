use crate::drivers::Results;
use crate::formatters::error::Result;
use crate::formatters::formatter::FormatterOptions;
use crate::formatters::table;
use async_trait::async_trait;
use lazy_static::lazy_static;
use prettytable::format::{FormatBuilder, TableFormat};

lazy_static! {
    pub static ref FORMAT_PLAIN: TableFormat = FormatBuilder::new().padding(0, 3).build();
}

/// A formatter for Unicode tables
#[derive(Debug, Default)]
pub(crate) struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "plain"
    }

    async fn format<'a>(
        &self,
        options: &mut FormatterOptions<'a>,
        results: &Results,
    ) -> Result<()> {
        table::format(*FORMAT_PLAIN, options, results).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::{MemoryQueryResult, Results, Value};
    use crate::formatters::Formatter;
    use crate::writers::Output;
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
        let output = &mut Output::default();
        let mut options = FormatterOptions {
            configuration: &mut configuration,
            elapsed: Duration::from_nanos(5678),
            output,
        };
        let formatter = Formatter;

        formatter.format(&mut options, &results).await?;

        let plain_output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r#"
            id   
            12,345   
            1 row (5.678µs)
        "#};
        assert_eq!(plain_output, expected);
        Ok(())
    }
}
