use crate::drivers::Value;
use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
use async_trait::async_trait;
use indexmap::IndexMap;
use serde_json::json;

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
        _ => return write_footer(options),
    };

    if !jsonl {
        write!(options.output, "[")?;
    }

    let columns: Vec<String> = query_result.columns.iter().map(|c| c.to_string()).collect();
    let rows_iter = query_result.rows.iter();
    for (i, row) in rows_iter.enumerate() {
        let mut json_row: IndexMap<&String, Option<Value>> = IndexMap::new();

        if i > 0 {
            if jsonl {
                writeln!(options.output)?;
            } else {
                writeln!(options.output, ",")?;
            }
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
        let json = json!(json_row);
        write!(options.output, "{}", json)?;
    }

    if jsonl {
        writeln!(options.output)?;
    } else {
        writeln!(options.output, "]")?;
    }

    write_footer(options)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::QueryResult;
    use crate::drivers::Results::{Execute, Query};
    use crate::drivers::Value;
    use crate::formatters::formatter::FormatterOptions;
    use crate::formatters::Formatter;
    use rustyline::ColorMode;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_format_execute() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color_mode: ColorMode::Disabled,
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
        let expected = "[{\"id\":1,\"data\":\"Ynl0ZXM=\"},\n{\"id\":2,\"data\":\"foo\"},\n{\"id\":3,\"data\":null}]\n3 rows (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }
}
