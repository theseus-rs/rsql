use crate::error::Result;
use crate::footer::write_footer;
use crate::formatter::FormatterOptions;
use crate::writers::Output;
use crate::Highlighter;
use async_trait::async_trait;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use rsql_drivers::Results;

/// A formatter for HTML
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "html"
    }

    async fn format(
        &self,
        options: &FormatterOptions,
        results: &Results,
        output: &mut Output,
    ) -> Result<()> {
        format_html(options, results, output).await
    }
}

pub(crate) async fn format_html(
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

    writer.write_event(Event::Start(BytesStart::new("table")))?;
    writer.write_event(Event::Start(BytesStart::new("thead")))?;
    writer.write_event(Event::Start(BytesStart::new("tr")))?;
    for column in &query_result.columns().await {
        writer.write_event(Event::Start(BytesStart::new("th")))?;
        writer.write_event(Event::Text(BytesText::new(column.as_str())))?;
        writer.write_event(Event::End(BytesEnd::new("th")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("tr")))?;
    writer.write_event(Event::End(BytesEnd::new("thead")))?;

    writer.write_event(Event::Start(BytesStart::new("tbody")))?;
    for row in &query_result.rows().await {
        writer.write_event(Event::Start(BytesStart::new("tr")))?;

        for data in row.iter() {
            match data {
                Some(value) => {
                    let string_value = value.to_string();
                    writer.write_event(Event::Start(BytesStart::new("td")))?;
                    writer.write_event(Event::Text(BytesText::new(string_value.as_str())))?;
                    writer.write_event(Event::End(BytesEnd::new("td")))?;
                }
                None => {
                    writer.write_event(Event::Empty(BytesStart::new("td")))?;
                }
            }
        }

        writer.write_event(Event::End(BytesEnd::new("tr")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("tbody")))?;
    writer.write_event(Event::End(BytesEnd::new("table")))?;

    let html_output = raw_output.to_string();
    let highlighter = Highlighter::new(options, "html");
    writeln!(output, "{}", highlighter.highlight(html_output.as_str())?)?;

    write_footer(options, results, output).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::formatter::FormatterOptions;
    use crate::writers::Output;
    use crate::Formatter;
    use indoc::indoc;
    use rsql_drivers::MemoryQueryResult;
    use rsql_drivers::Results::{Execute, Query};
    use rsql_drivers::Value;
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
            <table>
              <thead>
                <tr>
                  <th>id</th>
                  <th>data</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>1</td>
                  <td>Ynl0ZXM=</td>
                </tr>
                <tr>
                  <td>2</td>
                  <td>foo</td>
                </tr>
                <tr>
                  <td>3</td>
                  <td/>
                </tr>
              </tbody>
            </table>
            3 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
