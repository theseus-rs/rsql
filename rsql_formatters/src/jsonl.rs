use crate::Results;
use crate::error::Result;
use crate::formatter::FormatterOptions;
use crate::json::format_json;
use crate::writers::Output;
use async_trait::async_trait;

/// A formatter for JSONL
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "jsonl"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &mut Results,
        output: &mut Output,
    ) -> Result<()> {
        format_json(options, true, results, output).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Formatter;
    use crate::Results::Query;
    use crate::formatter::FormatterOptions;
    use crate::writers::Output;
    use indoc::indoc;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;

    #[tokio::test]
    async fn test_format_query() -> anyhow::Result<()> {
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
