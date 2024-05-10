use crate::error::Result;
use crate::footer::write_footer;
use crate::formatter::FormatterOptions;
use crate::highlighter::Highlighter;
use crate::writers::Output;
use crate::Results;
use crate::Results::Query;
use async_trait::async_trait;
use indexmap::IndexMap;
use rsql_drivers::Value;

/// A formatter for YAML
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "yaml"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &mut Results,
        output: &mut Output,
    ) -> Result<()> {
        format_yaml(options, results, output).await
    }
}

pub(crate) async fn format_yaml(
    options: &FormatterOptions,
    results: &mut Results,
    output: &mut Output,
) -> Result<()> {
    let query_result = match results {
        Query(query_result) => query_result,
        _ => return write_footer(options, results, 0, output).await,
    };

    let mut yaml_rows: Vec<IndexMap<&String, Value>> = Vec::new();
    let columns: Vec<String> = query_result.columns().await;
    let mut rows: u64 = 0;

    while let Some(row) = query_result.next().await {
        let mut yaml_row: IndexMap<&String, Value> = IndexMap::new();

        for (c, data) in row.into_iter().enumerate() {
            let column = columns.get(c).expect("column not found");
            if let Value::Bytes(_bytes) = data {
                let value = Value::String(data.to_string());
                yaml_row.insert(column, value);
            } else {
                yaml_row.insert(column, data.clone());
            }
        }

        yaml_rows.push(yaml_row);
        rows += 1;
    }

    let yaml = serde_yaml::to_string(&yaml_rows)?;
    let highlighter = Highlighter::new(options, "yaml");
    write!(output, "{}", highlighter.highlight(yaml.as_str())?)?;

    write_footer(options, results, rows, output).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::formatter::FormatterOptions;
    use crate::writers::Output;
    use crate::Formatter;
    use crate::Results::{Execute, Query};
    use indoc::indoc;
    use rsql_drivers::Value;
    use rsql_drivers::{MemoryQueryResult, Row};
    use std::time::Duration;

    #[tokio::test]
    async fn test_format_execute() -> anyhow::Result<()> {
        let options = FormatterOptions {
            color: false,
            elapsed: Duration::from_nanos(9),
            ..Default::default()
        };
        let output = &mut Output::default();

        let formatter = Formatter;
        formatter.format(&options, &mut Execute(1), output).await?;

        let output = output.to_string().replace("\r\n", "\n");
        let expected = "1 row (9ns)\n";
        assert_eq!(output, expected);
        Ok(())
    }

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
                Row::new(vec![Value::I64(1), Value::Bytes(b"bytes".to_vec())]),
                Row::new(vec![Value::I64(2), Value::String("foo".to_string())]),
                Row::new(vec![Value::I64(3), Value::Null]),
            ],
        )));
        let output = &mut Output::default();

        let formatter = Formatter;
        formatter
            .format(&options, &mut query_result, output)
            .await?;

        let output = output.to_string().replace("\r\n", "\n");
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
