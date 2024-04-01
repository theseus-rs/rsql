use crate::formatters::error::Result;
use crate::formatters::formatter::FormatterOptions;
use async_trait::async_trait;
use rsql_drivers::Results;

/// A formatter for JSONL
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "jsonl"
    }

    async fn format<'a>(
        &self,
        options: &mut FormatterOptions<'a>,
        results: &Results,
    ) -> Result<()> {
        crate::formatters::json::format_json(options, true, results).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use crate::formatters::formatter::FormatterOptions;
    use crate::formatters::Formatter;
    use crate::writers::Output;
    use indoc::indoc;
    use rsql_drivers::Results::Query;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;

    #[tokio::test]
    async fn test_format_query() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: false,
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
        let mut options = FormatterOptions {
            configuration,
            elapsed: Duration::from_nanos(9),
            output,
        };

        let formatter = Formatter;
        formatter.format(&mut options, &query_result).await?;

        let output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r#"
            {"id":1,"data":"Ynl0ZXM="}
            {"id":2,"data":"foo"}
            {"id":3,"data":null}
            3 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
