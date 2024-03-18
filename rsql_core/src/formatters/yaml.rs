use crate::drivers::Value;
use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
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
        _ => return Ok(()),
    };

    let mut yaml_rows: Vec<IndexMap<&String, Option<Value>>> = Vec::new();
    let columns: Vec<String> = query_result.columns.iter().map(|c| c.to_string()).collect();
    for row in &query_result.rows {
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
    write!(options.output, "{}", yaml)?;
    writeln!(options.output)?;

    write_footer(options)
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
    async fn test_format() -> anyhow::Result<()> {
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
        let expected = "- id: 1\n  data: Ynl0ZXM=\n- id: 2\n  data: foo\n- id: 3\n  data: null\n\n3 rows (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }
}
