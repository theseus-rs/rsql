use async_trait::async_trait;
use file_type::FileType;
use indexmap::IndexMap;
use polars::io::SerReader;
use polars::prelude::{IntoLazy, JsonReader};
use polars_sql::SQLContext;
use quick_xml::Reader;
use quick_xml::events::Event;
use rsql_driver::Error::{ConversionError, IoError};
use rsql_driver::{Result, UrlExtension};
use rsql_driver_polars::Connection;
use serde_json::{Number, Value, json};
use std::collections::HashMap;
use std::io::Cursor;
use std::num::NonZeroUsize;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "xml"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();

        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let json = {
            #[cfg(target_family = "wasm")]
            let xml = std::fs::read_to_string(&file_name)?;
            #[cfg(not(target_family = "wasm"))]
            let xml = tokio::fs::read_to_string(&file_name).await?;
            let value = xml_to_json(&xml)?;
            serde_json::to_string(&value).map_err(|error| IoError(error.to_string()))?
        };

        let ignore_errors = query_parameters
            .get("ignore_errors")
            .is_some_and(|value| value == "true");
        let infer_schema_length = match query_parameters.get("infer_schema_length") {
            Some(infer_schema_length) => {
                let length = infer_schema_length
                    .parse::<usize>()
                    .map_err(|error| ConversionError(error.to_string()))?;
                if length == 0 {
                    None
                } else {
                    NonZeroUsize::new(length)
                }
            }
            None => NonZeroUsize::new(100),
        };

        let cursor = Cursor::new(json.as_bytes());
        let data_frame = JsonReader::new(cursor)
            .infer_schema_len(infer_schema_length)
            .set_rechunk(true)
            .with_ignore_errors(ignore_errors)
            .finish()
            .map_err(|error| IoError(error.to_string()))?;

        let table_name = rsql_driver_polars::get_table_name(file_name)?;
        let context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type.media_types().contains(&"text/xml")
    }
}

/// Convert XML to JSON so that it can be read by Polars
#[doc(hidden)]
pub fn xml_to_json(xml: &str) -> Result<Value> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut stack: Vec<(String, IndexMap<String, Value>)> = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut element = IndexMap::new();
                // Store all attributes as key-value pairs where the name is prefixed with '@'
                for attribute in e.attributes() {
                    match attribute {
                        Ok(attribute) => {
                            let name = String::from_utf8_lossy(attribute.key.as_ref()).to_string();
                            let text =
                                String::from_utf8_lossy(attribute.value.as_ref()).to_string();
                            let value = infer_value(&text);
                            element.insert(format!("@{name}"), value);
                        }
                        Err(error) => return Err(IoError(error.to_string())),
                    }
                }
                stack.push((name, element));
            }
            Ok(Event::Text(e)) => {
                if let Some((_, map)) = stack.last_mut() {
                    let text = reader
                        .decoder()
                        .decode(e.as_ref())
                        .unwrap_or_default()
                        .into_owned();
                    if !text.is_empty() {
                        let value = infer_value(&text);
                        map.insert("#text".to_string(), value);
                    }
                }
            }
            Ok(Event::End(_)) => {
                if let Some((name, attributes)) = stack.pop() {
                    let value = match attributes.get("#text").cloned() {
                        Some(text) if attributes.len() == 1 => text,
                        _ if attributes.is_empty() => json!(null),
                        _ => json!(attributes),
                    };

                    if let Some((_, parent_map)) = stack.last_mut() {
                        match parent_map.get_mut(&name) {
                            Some(existing) => {
                                if let Value::Array(arr) = existing {
                                    arr.push(value);
                                } else {
                                    let prev = std::mem::replace(
                                        existing,
                                        json!([existing.clone(), value]),
                                    );
                                    parent_map.insert(name, json!([prev, value]));
                                }
                            }
                            None => {
                                parent_map.insert(name, value);
                            }
                        }
                    } else {
                        return Ok(json!({ name: value }));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(IoError(error.to_string())),
            _ => (),
        }
        buffer.clear();
    }

    Ok(json!({}))
}

fn infer_value(text: &str) -> Value {
    let text = text.trim();

    if let Ok(v) = text.parse::<u64>() {
        if text.starts_with('0') && text.len() > 1 {
            return Value::String(text.into());
        }
        return Value::Number(Number::from(v));
    }
    if let Ok(v) = text.parse::<f64>() {
        if text.starts_with('0') && !text.starts_with("0.") {
            return Value::String(text.into());
        }
        if let Some(val) = Number::from_f64(v) {
            return Value::Number(val);
        }
    }
    if let Ok(v) = text.parse::<bool>() {
        return Value::Bool(v);
    }

    Value::String(text.into())
}
