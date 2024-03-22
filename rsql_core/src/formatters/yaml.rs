use crate::drivers::Value;
use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
use crate::formatters::highlighter::Highlighter;
use async_trait::async_trait;
use indexmap::IndexMap;

/// A formatter for YAML
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "yaml"
    }

    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()> {
        format_yaml(options).await
    }
}

pub(crate) async fn format_yaml(options: &mut FormatterOptions<'_>) -> Result<()> {
    let query_result = match options.results {
        crate::drivers::Results::Query(query_result) => query_result,
        _ => return write_footer(options).await,
    };

    let mut yaml_rows: Vec<IndexMap<&String, Option<Value>>> = Vec::new();
    let columns: Vec<String> = query_result.columns().await;
    for row in &query_result.rows().await {
        let mut yaml_row: IndexMap<&String, Option<Value>> = IndexMap::new();

        for (c, data) in row.iter().enumerate() {
            let column = columns.get(c).expect("column not found");
            match data {
                Some(value) => {
                    if let Value::Bytes(_bytes) = value {
                        let value = Value::String(value.to_string());
                        yaml_row.insert(column, Some(value));
                    } else {
                        yaml_row.insert(column, Some(value.clone()));
                    }
                }
                None => {
                    yaml_row.insert(column, None);
                }
            }
        }

        yaml_rows.push(yaml_row);
    }

    let yaml = serde_yaml::to_string(&yaml_rows)?;
    let highlighter = Highlighter::new(options.configuration, "yaml");
    write!(options.output, "{}", highlighter.highlight(yaml.as_str())?)?;

    write_footer(options).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::MemoryQueryResult;
    use crate::drivers::Results::{Execute, Query};
    use crate::drivers::Value;
    use crate::formatters::formatter::FormatterOptions;
    use crate::formatters::Formatter;
    use indoc::indoc;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_format_execute() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: false,
            ..Default::default()
        };
        let output = &mut Cursor::new(Vec::new());
        let mut options = FormatterOptions {
            configuration,
            results: &Execute(1),
            elapsed: &std::time::Duration::from_nanos(9),
            output,
        };

        let formatter = Formatter;
        formatter.format(&mut options).await.unwrap();

        let output = String::from_utf8(output.get_ref().to_vec())?.replace("\r\n", "\n");
        let expected = "1 row (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }

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
            - id: 1
              data: Ynl0ZXM=
            - id: 2
              data: foo
            - id: 3
              data: null
            3 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
