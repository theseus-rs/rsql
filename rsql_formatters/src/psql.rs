use crate::error::Result;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use crate::{Results, table};
use async_trait::async_trait;
use tabled::settings::{Style, Theme};

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
        let theme = Theme::from_style(Style::psql());
        table::format(theme, options, results, output).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formatter;
    use crate::writers::Output;
    use indoc::indoc;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;

    fn query_result() -> Results {
        let query_result = MemoryQueryResult::new(
            vec!["id".to_string(), "value".to_string()],
            vec![
                vec![Value::I64(1234), Value::String("foo".to_string())],
                vec![Value::I64(5678), Value::String("bar".to_string())],
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
        let expected = indoc! {r"
               id   | value 
             -------+-------
              1,234 | foo   
              5,678 | bar   
             2 rows (9ns)
        "};
        assert_eq!(unicode_output, expected);
        Ok(())
    }
}
