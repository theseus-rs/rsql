use crate::drivers::Value;
use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
use crate::formatters::Highlighter;
use async_trait::async_trait;
use indexmap::IndexMap;
use serde_json::{json, to_string_pretty};

/// A formatter for JSON
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "json"
    }

    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()> {
        format_json(options, false).await
    }
}

pub(crate) async fn format_json(options: &mut FormatterOptions<'_>, jsonl: bool) -> Result<()> {
    let query_result = match options.results {
        crate::drivers::Results::Query(query_result) => query_result,
        _ => return write_footer(options).await,
    };

    let highlighter = Highlighter::new(options.configuration, "json");
    let mut json_rows: Vec<IndexMap<&String, Option<Value>>> = Vec::new();
    let columns: Vec<String> = query_result.columns().await;
    let rows = query_result.rows().await;
    for (i, row) in rows.iter().enumerate() {
        let mut json_row: IndexMap<&String, Option<Value>> = IndexMap::new();

        if i > 0 && jsonl {
            writeln!(options.output)?;
        }

        for (c, data) in row.iter().enumerate() {
            let column = columns.get(c).expect("column not found");
            match data {
                Some(value) => {
                    if let Value::Bytes(_bytes) = value {
                        let value = Value::String(value.to_string());
                        json_row.insert(column, Some(value));
                    } else {
                        json_row.insert(column, Some(value.clone()));
                    }
                }
                None => {
                    json_row.insert(column, None);
                }
            }
        }
        if !jsonl {
            json_rows.push(json_row);
        } else {
            let json = json!(json_row).to_string();
            write!(options.output, "{}", highlighter.highlight(json.as_str())?)?;
        }
    }

    if !jsonl {
        let json = to_string_pretty(&json!(json_rows))?;
        write!(options.output, "{}", highlighter.highlight(json.as_str())?)?;
    }

    writeln!(options.output)?;
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
          [
            {
              "id": 1,
              "data": "Ynl0ZXM="
            },
            {
              "id": 2,
              "data": "foo"
            },
            {
              "id": 3,
              "data": null
            }
          ]
          3 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
