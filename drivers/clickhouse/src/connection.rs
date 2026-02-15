use crate::metadata;
use crate::results::{ClickHouseQueryResult, parse_column_type};
use async_trait::async_trait;
use clickhouse::Client;
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{Metadata, QueryResult, Result, ToSql, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use url::Url;

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

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let values = rsql_driver::to_values(params);
        let mut query = self.client.query(sql);
        for value in &values {
            query = bind_clickhouse_value(query, value);
        }
        query
            .execute()
            .await
            .map_err(|error| IoError(error.to_string()))?;
        Ok(0)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let values = rsql_driver::to_values(params);
        let mut query = self.client.query(sql);
        for value in &values {
            query = bind_clickhouse_value(query, value);
        }
        let mut response = query
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

        let (columns, column_types): (Vec<String>, Vec<(Option<String>, String)>) = json_response
            .get("meta")
            .and_then(|value| value.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let column_name = item.get("name")?.as_str()?.to_string();
                        let column_type = item.get("type")?.as_str()?;
                        let (nullable, base_type) = parse_column_type(column_type);
                        Some((
                            column_name,
                            (nullable.map(ToString::to_string), base_type.to_string()),
                        ))
                    })
                    .unzip()
            })
            .unwrap_or_default();

        let data_rows: Vec<JsonValue> = json_response
            .get("data")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();

        let query_result = ClickHouseQueryResult::new(columns, column_types, data_rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }
}

fn bind_clickhouse_value(
    query: clickhouse::query::Query,
    value: &Value,
) -> clickhouse::query::Query {
    match value {
        Value::Null => query.bind(Option::<String>::None),
        Value::Bool(v) => query.bind(*v),
        Value::I8(v) => query.bind(*v),
        Value::I16(v) => query.bind(*v),
        Value::I32(v) => query.bind(*v),
        Value::I64(v) => query.bind(*v),
        Value::I128(v) => query.bind(v.to_string()),
        Value::U8(v) => query.bind(*v),
        Value::U16(v) => query.bind(*v),
        Value::U32(v) => query.bind(*v),
        Value::U64(v) => query.bind(*v),
        Value::U128(v) => query.bind(v.to_string()),
        Value::F32(v) => query.bind(*v),
        Value::F64(v) => query.bind(*v),
        Value::String(v) => query.bind(v.clone()),
        Value::Bytes(v) => query.bind(v.clone()),
        _ => query.bind(value.to_string()),
    }
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("url", &self.url)
            .field("client", &"<ClickHouse Client>")
            .finish()
    }
}
