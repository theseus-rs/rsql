use crate::formatters::delimited::format;
use crate::formatters::error::Result;
use crate::formatters::formatter::FormatterOptions;
use async_trait::async_trait;

/// A formatter for Column Separated Values (CSV)
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "csv"
    }

    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()> {
        format(options, b',').await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::MemoryQueryResult;
    use crate::drivers::Results::Query;
    use crate::drivers::Value;
    use crate::formatters::formatter::FormatterOptions;
    use crate::formatters::Formatter;
    use indoc::indoc;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
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
        let expected = indoc! {r#"
            "id","data"
            1,"Ynl0ZXM="
            2,"foo"
            3,""
            3 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
