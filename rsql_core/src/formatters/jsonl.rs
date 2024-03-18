use crate::formatters::error::Result;
use crate::formatters::formatter::FormatterOptions;
use async_trait::async_trait;

/// A formatter for JSONL
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "jsonl"
    }

    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()> {
        crate::formatters::json::format_json(options, true).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::QueryResult;
    use crate::drivers::Results::Query;
    use crate::drivers::Value;
    use crate::formatters::formatter::FormatterOptions;
    use crate::formatters::Formatter;
    use rustyline::ColorMode;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_format_query() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color_mode: ColorMode::Disabled,
            ..Default::default()
        };
        let query_result = Query(QueryResult {
            columns: vec!["id".to_string(), "data".to_string()],
            rows: vec![
                vec![Some(Value::I64(1)), Some(Value::Bytes(b"bytes".to_vec()))],
                vec![Some(Value::I64(2)), Some(Value::String("foo".to_string()))],
                vec![Some(Value::I64(3)), None],
            ],
        });
        let output = &mut Cursor::new(Vec::new());
        let mut options = FormatterOptions {
            configuration,
            results: &query_result,
            elapsed: &std::time::Duration::from_nanos(9),
            output,
        };

        let formatter = Formatter;
        formatter.format(&mut options).await.unwrap();

        let output = String::from_utf8(output.get_ref().to_vec())?.replace("\r\n", "\n");
        let expected = "{\"id\":1,\"data\":\"Ynl0ZXM=\"}\n{\"id\":2,\"data\":\"foo\"}\n{\"id\":3,\"data\":null}\n3 rows (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }
}
