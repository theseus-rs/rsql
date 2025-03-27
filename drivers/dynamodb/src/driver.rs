use crate::metadata;
use async_trait::async_trait;
use aws_config::{AppName, Region};
use aws_credential_types::Credentials;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use file_type::FileType;
use indexmap::IndexMap;
use rsql_driver::Error::{IoError, UnsupportedColumnType};
use rsql_driver::{MemoryQueryResult, Metadata, QueryResult, Result, Value};
use std::collections::HashMap;
use std::env;
use url::Url;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

/// Driver for AWS `DynamoDB` databases using the `aws-sdk-dynamodb` crate.
///
/// For a list of supported environment variables, see:
/// <https://docs.aws.amazon.com/sdkref/latest/guide/settings-reference.html#EVarSettings>
#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "dynamodb"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let parameters: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let sdk_config = aws_config::from_env().load().await;
        let mut config_builder = aws_sdk_dynamodb::config::Builder::from(&sdk_config);

        if let Ok(app_name) = AppName::new(PACKAGE_NAME) {
            config_builder = config_builder.app_name(app_name);
        }
        if let Some(credentials) = Self::credentials(&parsed_url, &parameters) {
            config_builder = config_builder.credentials_provider(credentials);
        }
        if let Some(region) = Self::region(&parameters) {
            config_builder = config_builder.region(region);
        }
        if parameters.contains_key("scheme") {
            let Some(endpoint_url) = Self::endpoint_url(&parsed_url, &parameters) else {
                return Err(IoError(
                    "Invalid DynamoDB URL; no endpoint url defined".to_string(),
                ));
            };
            config_builder = config_builder.endpoint_url(endpoint_url.as_str());
        }

        let config = config_builder.build();
        let client = Client::from_conf(config);

        let connection = Connection::new(url, client).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

impl Driver {
    /// Extracts the credentials from the URL and returns them as a `Credentials` object.
    /// If the URL does not contain credentials, it will look for S3 specific environment variables.
    fn credentials(parsed_url: &Url, parameters: &HashMap<String, String>) -> Option<Credentials> {
        let username = parsed_url.username();
        if username.is_empty() {
            return None;
        }

        let access_key = username.to_string();
        let secret_key = parsed_url.password()?.to_string();
        let session_token = parameters.get("session_token").cloned();
        Some(Credentials::from_keys(
            access_key,
            secret_key,
            session_token,
        ))
    }

    /// Extracts the region from the URL, or the `S3_REGION` environment variable and returns it as
    /// a `Region` object.
    fn region(parameters: &HashMap<String, String>) -> Option<Region> {
        parameters
            .get("region")
            .map(|region| Region::new(region.to_string()))
    }

    /// Extracts the endpoint URL from the URL and returns it as a string.
    fn endpoint_url(parsed_url: &Url, parameters: &HashMap<String, String>) -> Option<String> {
        if let Some(host) = parsed_url.host_str() {
            let port = parsed_url.port().unwrap_or(443);
            let scheme = parameters
                .get("scheme")
                .cloned()
                .unwrap_or("https".to_string());
            let endpoint_url = format!("{scheme}://{host}:{port}");
            Some(endpoint_url)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    url: String,
    client: Client,
}

impl Connection {
    #[expect(clippy::unused_async)]
    pub(crate) async fn new(url: &str, client: Client) -> Result<Connection> {
        let connection = Connection {
            url: url.to_string(),
            client,
        };
        Ok(connection)
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let mut rows: u64 = 0;
        let mut next_token = None;

        loop {
            let result = self
                .client
                .execute_statement()
                .statement(sql)
                .set_next_token(next_token.clone())
                .send()
                .await
                .map_err(|error| IoError(format!("{error:?}")))?;

            let items = result.items();
            let items_len = u64::try_from(items.len())?;
            rows = rows
                .checked_add(items_len)
                .ok_or(IoError("Integer overflow".to_string()))?;
            if let Some(token) = result.next_token() {
                next_token = Some(token.to_string());
            } else {
                break;
            }
        }

        Ok(rows)
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let mut columns = Vec::new();
        let mut rows = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let result = self
                .client
                .execute_statement()
                .statement(sql)
                .set_next_token(next_token.clone())
                .send()
                .await
                .map_err(|error| IoError(format!("{error:?}")))?;

            for item in result.items() {
                if columns.is_empty() {
                    for column_name in item.keys() {
                        columns.push(column_name.to_string());
                    }
                }
                let mut row_data = Vec::new();
                for (column_name, attribute) in item {
                    let value = Self::convert_to_value(column_name, attribute)?;
                    row_data.push(value);
                }
                rows.push(row_data);
            }

            if let Some(token) = result.next_token() {
                next_token = Some(token.to_string());
            } else {
                break;
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self, &self.client).await
    }
}

impl Connection {
    fn convert_to_value(column_name: &str, attribute: &AttributeValue) -> Result<Value> {
        let value = match attribute {
            AttributeValue::B(value) => {
                let value = value.as_ref().to_vec();
                Value::Bytes(value)
            }
            AttributeValue::Bool(value) => Value::Bool(*value),
            AttributeValue::Bs(values) => {
                let values = values
                    .iter()
                    .map(|value| Value::Bytes(value.as_ref().to_vec()))
                    .collect::<Vec<Value>>();
                Value::Array(values)
            }
            AttributeValue::L(values) => {
                let mut items = Vec::new();
                for item in values {
                    let value = Self::convert_to_value(column_name, item)?;
                    items.push(value);
                }
                Value::Array(items)
            }
            AttributeValue::M(values) => {
                let mut items = IndexMap::new();
                for (key, value) in values {
                    let key = Value::String(key.to_string());
                    let value = Self::convert_to_value(column_name, value)?;
                    items.insert(key, value);
                }
                Value::Map(items)
            }
            AttributeValue::N(value) => {
                if value.contains('.') {
                    let value: f64 = value
                        .parse()
                        .map_err(|error| IoError(format!("{error:?}")))?;
                    Value::F64(value)
                } else {
                    let value: i128 = value
                        .parse()
                        .map_err(|error| IoError(format!("{error:?}")))?;
                    Value::I128(value)
                }
            }
            AttributeValue::Ns(values) => {
                let mut items = Vec::new();
                for value in values {
                    let value = if value.contains('.') {
                        let value: f64 = value
                            .parse()
                            .map_err(|error| IoError(format!("{error:?}")))?;
                        Value::F64(value)
                    } else {
                        let value: i128 = value
                            .parse()
                            .map_err(|error| IoError(format!("{error:?}")))?;
                        Value::I128(value)
                    };
                    items.push(value);
                }
                Value::Array(items)
            }
            AttributeValue::Null(_value) => Value::Null,
            AttributeValue::S(value) => Value::String(value.to_string()),
            AttributeValue::Ss(values) => {
                let values = values
                    .iter()
                    .map(|value| Value::String(value.to_string()))
                    .collect::<Vec<Value>>();
                Value::Array(values)
            }
            _ => {
                return Err(UnsupportedColumnType {
                    column_name: column_name.to_string(),
                    column_type: format!("{attribute:?}"),
                });
            }
        };
        Ok(value)
    }
}
