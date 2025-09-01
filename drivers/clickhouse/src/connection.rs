use crate::metadata;
use async_trait::async_trait;
use clickhouse::Client;
use jiff::civil::{Date, DateTime};
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{MemoryQueryResult, Metadata, QueryResult, Result, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use url::Url;
use uuid::Uuid;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Connection {
    url: String,
    client: Client,
}

impl Connection {
    pub async fn new(url: &str) -> Result<Self> {
        let parsed_url = Url::parse(url).map_err(|error| InvalidUrl(error.to_string()))?;
        let parameters: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

        let host = parsed_url.host_str().unwrap_or("localhost");
        let port = parsed_url.port().unwrap_or(8123);
        let username = parsed_url.username();
        let password = parsed_url.password();
        let database = parsed_url.path().trim_start_matches('/');

        let scheme = parameters
            .get("scheme")
            .cloned()
            .unwrap_or("https".to_string());
        let client_url = format!("{scheme}://{}:{}", host, port);

        let mut client = Client::default()
            .with_product_info(PACKAGE_NAME, PACKAGE_VERSION)
            .with_url(client_url);

        if !username.is_empty() {
            client = client.with_user(username);
        }

        if let Some(pwd) = password {
            client = client.with_password(pwd);
        }

        if let Some(jwt) = parameters.get("access_token") {
            client = client.with_access_token(jwt.clone());
        }

        if !database.is_empty() {
            client = client.with_database(database.to_string());
        }

        Ok(Self {
            url: url.to_string(),
            client,
        })
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        self.client
            .query(sql)
            .execute()
            .await
            .map_err(|error| IoError(error.to_string()))?;
        Ok(0)
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let mut response = self
            .client
            .query(sql)
            .fetch_bytes("JSON")
            .map_err(|error| IoError(error.to_string()))?;

        let bytes = response
            .collect()
            .await
            .map_err(|error| IoError(format!("Failed to read response: {error:?}")))?;
        let json_str = String::from_utf8(bytes.to_vec())
            .map_err(|error| IoError(format!("Failed to convert response to string: {error:?}")))?;
        let json_response: JsonValue = serde_json::from_str(&json_str)
            .map_err(|error| IoError(format!("Failed to parse JSON response: {error:?}")))?;

        let (columns, column_types): (Vec<String>, Vec<(Option<&str>, &str)>) = json_response
            .get("meta")
            .and_then(|value| value.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let column_name = item.get("name")?.as_str()?.to_string();
                        let column_type = item.get("type")?.as_str()?;
                        let column_type = parse_column_type(column_type);
                        Some((column_name, column_type))
                    })
                    .unzip()
            })
            .unwrap_or_default();

        let mut rows = Vec::new();
        if let Some(data_array) = json_response.get("data").and_then(|value| value.as_array()) {
            for row_json in data_array {
                let Some(row_data) = row_json.as_object() else {
                    continue;
                };
                let mut row = Vec::with_capacity(columns.len());
                for (column_name, column_type) in columns.iter().zip(column_types.iter()) {
                    let Some(value) = row_data.get(column_name) else {
                        row.push(Value::Null);
                        continue;
                    };
                    let converted_value = convert_json_to_value(column_type, value)?;
                    row.push(converted_value);
                }
                rows.push(row);
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }
}

fn parse_column_type(column_type: &str) -> (Option<&str>, &str) {
    if let Some(start) = column_type.find('(')
        && let Some(end) = column_type.rfind(')')
    {
        let container = &column_type[..start];
        let inner = &column_type[start + 1..end];
        return (Some(container), inner);
    }
    (None, column_type)
}

fn convert_json_to_value(
    column_type: &(Option<&str>, &str),
    json_value: &JsonValue,
) -> Result<Value> {
    let value = match column_type {
        (Some("Array"), column_type) => {
            if let Some(array) = json_value.as_array() {
                let values = array
                    .iter()
                    .map(|item| {
                        let column_type = parse_column_type(column_type);
                        convert_json_to_value(&column_type, item)
                    })
                    .collect::<Result<Vec<Value>>>()?;
                Value::Array(values)
            } else {
                Value::Null
            }
        }
        (_, "Nothing") => Value::Null,
        (_, "Bool") => {
            if let Some(b) = json_value.as_bool() {
                Value::Bool(b)
            } else {
                Value::Null
            }
        }
        (_, "Int8") => {
            if let Some(i) = json_value.as_i64() {
                Value::I8(i as i8)
            } else {
                Value::Null
            }
        }
        (_, "Int16") => {
            if let Some(i) = json_value.as_i64() {
                Value::I16(i as i16)
            } else {
                Value::Null
            }
        }
        (_, "Int32") => {
            if let Some(i) = json_value.as_i64() {
                Value::I32(i as i32)
            } else {
                Value::Null
            }
        }
        (_, "Int64") => {
            if let Some(s) = json_value.as_str() {
                let value = s
                    .parse::<i64>()
                    .map_err(|error| IoError(error.to_string()))?;
                Value::I64(value)
            } else {
                Value::Null
            }
        }
        (_, "Int128") => {
            if let Some(s) = json_value.as_str() {
                let value = s
                    .parse::<i128>()
                    .map_err(|error| IoError(error.to_string()))?;
                Value::I128(value)
            } else {
                Value::Null
            }
        }
        (_, "UInt8") => {
            if let Some(i) = json_value.as_u64() {
                Value::U8(i as u8)
            } else {
                Value::Null
            }
        }
        (_, "UInt16") => {
            if let Some(i) = json_value.as_u64() {
                Value::U16(i as u16)
            } else {
                Value::Null
            }
        }
        (_, "UInt32") => {
            if let Some(i) = json_value.as_u64() {
                Value::U32(i as u32)
            } else {
                Value::Null
            }
        }
        (_, "UInt64") => {
            if let Some(s) = json_value.as_str() {
                let value = s
                    .parse::<u64>()
                    .map_err(|error| IoError(error.to_string()))?;
                Value::U64(value)
            } else {
                Value::Null
            }
        }
        (_, "UInt128") => {
            if let Some(s) = json_value.as_str() {
                let value = s
                    .parse::<u128>()
                    .map_err(|error| IoError(error.to_string()))?;
                Value::U128(value)
            } else {
                Value::Null
            }
        }
        (_, "Float32") => {
            if let Some(f) = json_value.as_f64() {
                Value::F32(f as f32)
            } else {
                Value::Null
            }
        }
        (_, "Float64") => {
            if let Some(f) = json_value.as_f64() {
                Value::F64(f)
            } else {
                Value::Null
            }
        }
        (_, "String" | "FixedString") => {
            if let Some(s) = json_value.as_str() {
                Value::String(s.to_string())
            } else {
                Value::Null
            }
        }
        (_, "Date") => {
            if let Some(s) = json_value.as_str() {
                let date =
                    Date::strptime("%Y-%m-%d", s).map_err(|error| IoError(error.to_string()))?;
                Value::Date(date)
            } else {
                Value::Null
            }
        }
        (_, "DateTime") => {
            if let Some(s) = json_value.as_str() {
                let date_time = DateTime::strptime("%Y-%m-%d %H:%M:%S", s)
                    .map_err(|error| IoError(error.to_string()))?;
                Value::DateTime(date_time)
            } else {
                Value::Null
            }
        }
        (_, "UUID") => {
            if let Some(s) = json_value.as_str() {
                let uuid = Uuid::parse_str(s).map_err(|error| IoError(error.to_string()))?;
                Value::Uuid(uuid)
            } else {
                Value::Null
            }
        }
        _ => {
            return Err(IoError(format!("Unsupported data type: {column_type:?}")));
        }
    };
    Ok(value)
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("url", &self.url)
            .field("client", &"<ClickHouse Client>")
            .finish()
    }
}
