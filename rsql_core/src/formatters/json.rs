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
        _ => return Ok(()),
    };

    if !jsonl {
        writeln!(options.output, "[")?;
    }

    let columns: Vec<String> = query_result.columns.iter().map(|c| c.to_string()).collect();
    let rows_iter = query_result.rows.iter();
    for (i, row) in rows_iter.enumerate() {
        let mut csv_row: IndexMap<&String, Option<serde_json::Value>> = IndexMap::new();

        if i > 0 {
            if !jsonl {
                write!(options.output, ",")?;
            }
            writeln!(options.output)?;
        }

        for (c, data) in row.iter().enumerate() {
            let column = columns.get(c).expect("column not found");
            match data {
                Some(value) => {
                    let value: serde_json::Value = match value {
                        Value::Bool(value) => json!(value),
                        Value::Bytes(_bytes) => json!(value.to_string()),
                        Value::I8(value) => json!(value),
                        Value::I16(value) => json!(value),
                        Value::I32(value) => json!(value),
                        Value::I64(value) => json!(value),
                        Value::U8(value) => json!(value),
                        Value::U16(value) => json!(value),
                        Value::U32(value) => json!(value),
                        Value::U64(value) => json!(value),
                        Value::F32(value) => json!(value),
                        Value::F64(value) => json!(value),
                        Value::String(value) => json!(value),
                        Value::Date(value) => json!(value),
                        Value::Time(value) => json!(value),
                        Value::DateTime(value) => json!(value),
                        Value::Uuid(value) => json!(value.to_string()),
                        Value::Json(value) => value.clone(),
                    };
                    csv_row.insert(column, Some(value));
                }
                None => {
                    csv_row.insert(column, None);
                }
            }
        }
        let json = json!(csv_row);
        write!(options.output, "{}", json)?;
    }

    writeln!(options.output)?;

    if !jsonl {
        writeln!(options.output, "]")?;
    }

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
        let expected = "[\n{\"id\":1,\"data\":\"Ynl0ZXM=\"},\n{\"id\":2,\"data\":\"foo\"},\n{\"id\":3,\"data\":null}\n]\n3 rows (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }
}
