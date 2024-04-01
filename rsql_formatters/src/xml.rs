use crate::error::Result;
use crate::footer::write_footer;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use crate::Highlighter;
use async_trait::async_trait;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use rsql_drivers::Results;

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
        results: &Results,
        output: &mut Output,
    ) -> Result<()> {
        format_xml(options, results, output).await
    }
}

pub(crate) async fn format_xml(
    options: &FormatterOptions,
    results: &Results,
    output: &mut Output,
) -> Result<()> {
    let query_result = match results {
        Results::Query(query_result) => query_result,
        _ => return write_footer(options, results, output).await,
    };

    let mut raw_output = Output::default();
    let mut writer = Writer::new_with_indent(&mut raw_output, b' ', 2);

    writer.write_event(Event::Start(BytesStart::new("results")))?;
    let columns: Vec<String> = query_result.columns().await;
    for row in &query_result.rows().await {
        writer.write_event(Event::Start(BytesStart::new("row")))?;
        for (c, data) in row.iter().enumerate() {
            let column = columns.get(c).expect("column not found");

            match data {
                Some(value) => {
                    let string_value = value.to_string();
                    writer.write_event(Event::Start(BytesStart::new(column)))?;
                    writer.write_event(Event::Text(BytesText::new(string_value.as_str())))?;
                    writer.write_event(Event::End(BytesEnd::new(column)))?;
                }
                None => {
                    writer.write_event(Event::Empty(BytesStart::new(column)))?;
                }
            }
        }
        writer.write_event(Event::End(BytesEnd::new("row")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("results")))?;

    let xml_output = raw_output.to_string();
    let highlighter = Highlighter::new(options, "xml");
    writeln!(output, "{}", highlighter.highlight(xml_output.as_str())?)?;

    write_footer(options, results, output).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::formatter::FormatterOptions;
    use crate::Formatter;
    use indoc::indoc;
    use rsql_drivers::Results::{Execute, Query};
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
        formatter.format(&options, &Execute(1), output).await?;

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
        let query_result = Query(Box::new(MemoryQueryResult::new(
            vec!["id".to_string(), "data".to_string()],
            vec![
                vec![Some(Value::I64(1)), Some(Value::Bytes(b"bytes".to_vec()))],
                vec![Some(Value::I64(2)), Some(Value::String("foo".to_string()))],
                vec![Some(Value::I64(3)), None],
            ],
        )));
        let output = &mut Output::default();

        let formatter = Formatter;
        formatter.format(&options, &query_result, output).await?;

        let output = output.to_string().replace("\r\n", "\n");
        let expected = indoc! {r#"
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
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
