use crate::drivers::Value;
use crate::formatters::error::Result;
use crate::formatters::footer::write_footer;
use crate::formatters::formatter::FormatterOptions;
use async_trait::async_trait;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

/// A formatter for HTML
#[derive(Debug, Default)]
pub struct Formatter;

#[async_trait]
impl crate::formatters::Formatter for Formatter {
    fn identifier(&self) -> &'static str {
        "html"
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

    let mut writer = Writer::new(&mut options.output);

    writer.write_event(Event::Start(BytesStart::new("table")))?;
    writeln!(writer.get_mut())?;

    write!(writer.get_mut(), "  ")?;
    writer.write_event(Event::Start(BytesStart::new("thead")))?;
    writeln!(writer.get_mut())?;
    write!(writer.get_mut(), "    ")?;
    writer.write_event(Event::Start(BytesStart::new("tr")))?;
    for column in &query_result.columns().await {
        writer.write_event(Event::Start(BytesStart::new("th")))?;
        writer.write_event(Event::Text(BytesText::new(column.as_str())))?;
        writer.write_event(Event::End(BytesEnd::new("th")))?;
    }
    writer.write_event(Event::End(BytesEnd::new("tr")))?;
    writeln!(writer.get_mut())?;
    write!(writer.get_mut(), "  ")?;
    writer.write_event(Event::End(BytesEnd::new("thead")))?;
    writeln!(writer.get_mut())?;

    write!(writer.get_mut(), "  ")?;
    writer.write_event(Event::Start(BytesStart::new("tbody")))?;
    writeln!(writer.get_mut())?;

    for row in &query_result.rows().await {
        write!(writer.get_mut(), "    ")?;
        writer.write_event(Event::Start(BytesStart::new("tr")))?;

        for data in row.iter() {
            writer.write_event(Event::Start(BytesStart::new("td")))?;
            match data {
                Some(value) => {
                    let string_value = if let Value::Bytes(_bytes) = value {
                        let value = Value::String(value.to_string());
                        value.to_string()
                    } else {
                        value.to_string()
                    };
                    writer.write_event(Event::Text(BytesText::new(string_value.as_str())))?;
                }
                None => {}
            }

            writer.write_event(Event::End(BytesEnd::new("td")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("tr")))?;
        writeln!(writer.get_mut())?;
    }

    write!(writer.get_mut(), "  ")?;
    writer.write_event(Event::End(BytesEnd::new("tbody")))?;
    writeln!(writer.get_mut())?;

    writer.write_event(Event::End(BytesEnd::new("table")))?;
    writeln!(writer.get_mut())?;

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
            <table>
              <thead>
                <tr><th>id</th><th>data</th></tr>
              </thead>
              <tbody>
                <tr><td>1</td><td>Ynl0ZXM=</td></tr>
                <tr><td>2</td><td>foo</td></tr>
                <tr><td>3</td><td></td></tr>
              </tbody>
            </table>
            3 rows (9ns)
        "#};
        assert_eq!(output, expected);
        Ok(())
    }
}
