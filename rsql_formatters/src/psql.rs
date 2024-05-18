use crate::error::Result;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use crate::{table, Results};
use async_trait::async_trait;
use lazy_static::lazy_static;
use prettytable::format::{FormatBuilder, LinePosition, LineSeparator, TableFormat};

lazy_static! {
    pub static ref FORMAT_UNICODE: TableFormat = FormatBuilder::new()
        .column_separator('|')
        .separators(
            &[LinePosition::Title],
            LineSeparator::new('-', '+', '-', '-')
        )
        .padding(1, 1)
        .build();
}

/// A formatter for psql tables
#[derive(Debug, Default)]
pub(crate) struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "psql"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &mut Results,
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
    use rsql_drivers::{MemoryQueryResult, Row, Value};
    use std::time::Duration;

    fn query_result() -> Results {
        let query_result = MemoryQueryResult::new(
            vec!["id".to_string(), "value".to_string()],
            vec![
                Row::new(vec![Value::I64(1234), Value::String("foo".to_string())]),
                Row::new(vec![Value::I64(5678), Value::String("bar".to_string())]),
            ],
        );

        Results::Query(Box::new(query_result))
    }

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
        let options = FormatterOptions {
            color: false,
            elapsed: Duration::from_nanos(9),
            ..Default::default()
        };
        let mut results = query_result();
        let output = &mut Output::default();
        let formatter = Formatter;

        formatter.format(&options, &mut results, output).await?;

        let unicode_output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r#"
               id   | value 
             -------+-------
              1,234 | foo 
              5,678 | bar 
             2 rows (9ns)
        "#};
        assert_eq!(unicode_output, expected);
        Ok(())
    }
}
