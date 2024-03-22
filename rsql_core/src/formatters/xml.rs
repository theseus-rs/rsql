use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
use crate::formatters::Highlighter;
use async_trait::async_trait;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

/// A formatter for XML
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "xml"
    }

    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()> {
        format_xml(options).await
    }
}

pub(crate) async fn format_xml(options: &mut FormatterOptions<'_>) -> Result<()> {
    let query_result = match options.results {
        crate::drivers::Results::Query(query_result) => query_result,
        _ => return write_footer(options).await,
    };

    let mut output = Vec::new();
    let mut writer = Writer::new_with_indent(&mut output, b' ', 2);

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

    let xml_output = String::from_utf8(output.clone())?;
    let highlighter = Highlighter::new(options.configuration, "xml");
    writeln!(
        &mut options.output,
        "{}",
        highlighter.highlight(xml_output.as_str())?
    )?;

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
