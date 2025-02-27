use crate::Results::{Execute, Query};
use crate::error::Result;
use crate::footer::write_footer;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use crate::{Highlighter, Results};
use async_trait::async_trait;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

/// A formatter for XML
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "xml"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &mut Results,
        output: &mut Output,
    ) -> Result<()> {
        format_xml(options, results, output).await
    }
}

pub(crate) async fn format_xml(
    options: &FormatterOptions,
    results: &mut Results,
    output: &mut Output,
) -> Result<()> {
    let query_result = match results {
        Query(query_result) => query_result,
        Execute(_) => return write_footer(options, results, 0, output).await,
    };

    let mut raw_output = Output::default();
    let mut writer = Writer::new_with_indent(&mut raw_output, b' ', 2);

    writer.write_event(Event::Start(BytesStart::new("results")))?;
    let columns: Vec<String> = query_result.columns().await;
    let mut rows: u64 = 0;

    while let Some(row) = query_result.next().await {
        writer.write_event(Event::Start(BytesStart::new("row")))?;
        for (c, data) in row.into_iter().enumerate() {
            let column = columns.get(c).expect("column not found");

            if data.is_null() {
                writer.write_event(Event::Empty(BytesStart::new(column)))?;
            } else {
                let string_value = data.to_string();
                writer.write_event(Event::Start(BytesStart::new(column)))?;
                writer.write_event(Event::Text(BytesText::new(string_value.as_str())))?;
                writer.write_event(Event::End(BytesEnd::new(column)))?;
            }
        }
        writer.write_event(Event::End(BytesEnd::new("row")))?;
        rows += 1;
    }
    writer.write_event(Event::End(BytesEnd::new("results")))?;

    let xml_output = raw_output.to_string();
    let highlighter = Highlighter::new(options, "xml");
    writeln!(output, "{}", highlighter.highlight(xml_output.as_str())?)?;

    write_footer(options, results, rows, output).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Formatter;
    use crate::Results::{Execute, Query};
    use crate::formatter::FormatterOptions;
    use indoc::indoc;
    use rsql_drivers::{MemoryQueryResult, Value};
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
        let expected = indoc! {r"
            <results>
              <row>
                <id>1</id>
                <data>Ynl0ZXM=</data>
              </row>
              <row>
                <id>2</id>
                <data>foo</data>
              </row>
              <row>
                <id>3</id>
                <data/>
              </row>
            </results>
            3 rows (9ns)
        "};
        assert_eq!(output, expected);
        Ok(())
    }
}
