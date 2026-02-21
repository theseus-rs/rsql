use crate::Results;
use crate::Results::Query;
use crate::error::Result;
use crate::footer::write_footer;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use async_trait::async_trait;
use rsql_drivers::{QueryResult, Value, ValueFormatter};
use tabled::tables::ExtendedTable;

/// A formatter for expanded tables
#[derive(Debug, Default)]
pub(crate) struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "expanded"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &mut Results,
        output: &mut Output,
    ) -> Result<()> {
        let mut rows: u64 = 0;

        if let Query(query_result) = results {
            if query_result.columns().is_empty() {
                write_footer(options, results, 0, output).await?;
                return Ok(());
            }
            let mut data: Vec<Vec<String>> = Vec::new();
            data.push(query_result.columns().to_vec());
            rows = process_data(options, query_result, &mut data).await?;
            let locale = options.locale.clone();
            let table = ExtendedTable::from(data).template(move |index| {
                let value_formatter = ValueFormatter::new(&locale);
                let record = value_formatter.format_integer(index + 1);
                t!("expanded_record", locale = &locale, record = record).to_string()
            });

            writeln!(output, "{table}")?;
        }

        write_footer(options, results, rows, output).await?;
        Ok(())
    }
}

async fn process_data(
    options: &FormatterOptions,
    query_result: &mut Box<dyn QueryResult>,
    data: &mut Vec<Vec<String>>,
) -> Result<u64> {
    let mut raw_rows: Vec<Vec<Value>> = Vec::new();
    while let Some(row) = query_result.next().await {
        raw_rows.push(row.clone());
    }

    let value_formatter = ValueFormatter::new(options.locale.as_str());
    let mut rows: u64 = 0;
    for row in raw_rows {
        let mut row_data = Vec::new();

        for data in row {
            let data = match data {
                Value::Null => "NULL".to_string(),
                _ => data.to_formatted_string(&value_formatter),
            };

            row_data.push(data);
        }

        rows += 1;
        data.push(row_data);
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formatter;
    use crate::writers::Output;
    use indoc::indoc;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;

    fn query_result() -> Results {
        let query_result = MemoryQueryResult::new(
            vec!["id".to_string(), "value".to_string()],
            vec![
                vec![Value::I64(1234), Value::String("foo".to_string())],
                vec![Value::I64(5678), Value::Null],
            ],
        );

        Query(Box::new(query_result))
    }

    #[tokio::test]
    async fn test_format() -> anyhow::Result<()> {
        let options = FormatterOptions {
            color: false,
            elapsed: Duration::from_nanos(9),
            ..Default::default()
        };
        let mut results = query_result();
        let output = &mut Output::default();
        let formatter = Formatter;

        formatter.format(&options, &mut results, output).await?;

        let expanded_output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r"
            -[ RECORD 1 ]-
            id    | 1,234
            value | foo
            -[ RECORD 2 ]-
            id    | 5,678
            value | NULL
            2 rows (9ns)
        "};
        assert_eq!(expanded_output, expected);
        Ok(())
    }
}
