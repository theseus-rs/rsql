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
use tokio::fs;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "xml"
    }

    async fn connect(
        &self,
        url: &str,
        _password: Option<String>,
    ) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();

        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let json = {
            let xml = fs::read_to_string(&file_name).await?;
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
        let mut context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type.media_types().contains(&"text/xml")
    }
}

/// Convert XML to JSON so that it can be read by Polars
fn xml_to_json(xml: &str) -> Result<Value> {
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
                    let text = e.unescape().unwrap_or_default().into_owned();
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

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use rsql_driver::{Driver, Value};
    use rsql_driver_test_utils::dataset_url;

    fn database_url() -> String {
        dataset_url("xml", "users.xml")
    }

    #[tokio::test]
    async fn test_driver_connect() -> Result<()> {
        let database_url = database_url();
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url, None).await?;
        assert_eq!(&database_url, connection.url());
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> Result<()> {
        let database_url = database_url();
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url, None).await?;

        let mut query_result = connection
            .query(indoc! {r"
                    WITH cte_user AS (
                        SELECT unnest(data.user) FROM users
                    )
                    SELECT user.* FROM cte_user
                "})
            .await?;

        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
        );
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    #[test]
    fn test_xml_to_json() -> Result<()> {
        let xml = indoc! {r#"
            <data foo="42">
                <user score="1.234">
                    <id>1</id>
                    <name>John Doe</name>
                    <email secure="false">john.doe@none.com</email>
                </user>
                <user>
                    <name>Jane Smith</name>
                    <id>2</id>
                </user>
            </data>
        "#};

        let value = xml_to_json(xml)?;
        let _json = serde_json::to_string(&value);
        let data = value.get("data").expect("Expected data object");
        let foo = data.get("@foo").expect("Expected foo attribute");
        let foo_value = foo
            .as_i64()
            .expect("Expected foo attribute to be an integer");
        assert_eq!(foo_value, 42);
        let user = data.get("user").expect("Expected user value");
        let user = user.as_array().expect("Expected user value to be an array");
        assert_eq!(user.len(), 2);
        let user1 = user.first().expect("Expected user 1");
        let score = user1
            .get("@score")
            .expect("Expected score attribute")
            .as_f64()
            .expect("Expected score attribute to be a float");
        let diff = score - 1.234;
        assert!(diff.abs() < 0.01f64);
        let user1_id = user1
            .get("id")
            .expect("Expected id")
            .as_i64()
            .expect("Expected id to be an integer");
        let user1_name = user1
            .get("name")
            .expect("Expected name")
            .as_str()
            .expect("Expected name to be a string");
        assert_eq!(user1_id, 1);
        assert_eq!(user1_name, "John Doe");

        // Test element with text and an attribute
        let user1_email = user1.get("email").expect("Expected email");
        let user1_email_secure = user1_email
            .get("@secure")
            .expect("Expected secure attribute")
            .as_bool()
            .expect("Expected secure attribute to be a boolean");
        let user1_email = user1_email
            .get("#text")
            .expect("Expected email text")
            .as_str()
            .expect("Expected email text to be a string");
        assert!(!user1_email_secure);
        assert_eq!(user1_email, "john.doe@none.com");

        let user2 = user.last().expect("Expected user 2");
        let user2_id = user2
            .get("id")
            .expect("Expected id")
            .as_i64()
            .expect("Expected id to be an integer");
        let user2_name = user2
            .get("name")
            .expect("Expected name")
            .as_str()
            .expect("Expected name to be a string");
        assert_eq!(user2_id, 2);
        assert_eq!(user2_name, "Jane Smith");
        Ok(())
    }
}
