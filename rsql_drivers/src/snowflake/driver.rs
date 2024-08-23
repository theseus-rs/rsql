use crate::{
    snowflake::SnowflakeError, MemoryQueryResult, Metadata, QueryResult, Result, Row, Value,
};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use jwt_simple::prelude::{Claims, Duration, RS256KeyPair, RS256PublicKey, RSAKeyPairLike};
use reqwest::header::HeaderMap;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{error, info};
use url::Url;

const DATE_FORMATS: (&str, &str) = ("YYYY-MM-DD", "%Y-%m-%d");
const TIME_FORMATS: (&str, &str) = ("HH24:MI:SS.FF", "%H:%M:%S.%f");
const DATETIME_NO_TZ_FORMATS: (&str, &str) = ("YYYY-MM-DDTHH24:MI:SS.FF", "%Y-%m-%dT%H:%M:%S.%f");
const DATETIME_TZ_FORMATS: (&str, &str) =
    ("YYYY-MM-DDTHH24:MI:SS.FFTZHTZM", "%Y-%m-%dT%H:%M:%S.%f%:z");

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "snowflake"
    }

    async fn connect(
        &self,
        url: String,
        _password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        Ok(Box::new(SnowflakeConnection::new(&url)?))
    }
}

#[derive(Debug)]
pub(crate) struct SnowflakeConnection {
    base_url: String,
    issuer: String,
    subject: String,
    key_pair: RS256KeyPair,
    jwt_expires_at: DateTime<Utc>,
    client: Mutex<reqwest::Client>,
}

impl SnowflakeConnection {
    /// Generate a fingerprint for a public key
    /// Doing this manually since `jwt_simple` uses url-safe base64 when standard is required
    ///
    /// # Errors
    /// Errors if the public key is malformed
    fn public_key_fingerprint(public_key: &str) -> Result<String> {
        let public_key =
            RS256PublicKey::from_pem(public_key).map_err(|_| SnowflakeError::MalformedPublicKey)?;
        let pub_key_der = public_key
            .to_der()
            .map_err(|_| SnowflakeError::MalformedPublicKey)?;
        let mut hasher = Sha256::new();
        hasher.update(&pub_key_der);
        Ok(STANDARD.encode(hasher.finalize()))
    }

    /// Establish a new connection to Snowflake
    ///
    /// # Errors
    /// Errors if the URL is malformed, the account, user, private key, or public key are missing,
    /// or if the private key or public key files are missing or malformed
    pub(crate) fn new(url: &str) -> Result<SnowflakeConnection> {
        let parsed_url = Url::parse(url)?;
        let query_params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let base_url = parsed_url
            .host_str()
            .ok_or(SnowflakeError::MissingAccount)?
            .to_string();
        let account = base_url
            .split('.')
            .next()
            .ok_or(SnowflakeError::MissingAccount)?
            .to_string();
        let user = query_params
            .get("user")
            .ok_or(SnowflakeError::MissingUser)?
            .to_string();
        let private_key_file = query_params
            .get("private_key_file")
            .ok_or(SnowflakeError::MissingPrivateKey)?
            .to_string();
        let public_key_file = query_params
            .get("public_key_file")
            .ok_or(SnowflakeError::MissingPublicKey)?
            .to_string();

        let private_key = std::fs::read_to_string(private_key_file)
            .map_err(|_| SnowflakeError::MissingPrivateKey)?;
        let public_key = std::fs::read_to_string(public_key_file)
            .map_err(|_| SnowflakeError::MissingPublicKey)?;
        let key_pair =
            RS256KeyPair::from_pem(&private_key).map_err(|_| SnowflakeError::MissingPrivateKey)?;

        let fingerprint = Self::public_key_fingerprint(&public_key)?;
        let issuer = format!("{account}.{user}.SHA256:{fingerprint}");
        let subject = format!("{account}.{user}");

        let base_url = format!("https://{base_url}/api/v2/statements");
        let jwt_expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        let client = Mutex::new(Self::new_client(&issuer, &subject, &key_pair)?);

        Ok(Self {
            base_url,
            issuer,
            subject,
            key_pair,
            jwt_expires_at,
            client,
        })
    }

    fn new_client(issuer: &str, subject: &str, key_pair: &RS256KeyPair) -> Result<reqwest::Client> {
        let claims = Claims::create(Duration::from_hours(1))
            .with_issuer(issuer)
            .with_subject(subject);

        let token = key_pair
            .sign(claims)
            .map_err(|_| SnowflakeError::JwtSignature)?;

        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_owned(),
            format!("Bearer {}", token.clone()),
        );
        headers.insert("Content-Type".to_owned(), "application/json".to_owned());
        headers.insert(
            "X-Snowflake-Authorization-Token-Type".to_owned(),
            "KEYPAIR_JWT".to_owned(),
        );
        let header_map: HeaderMap = (&headers)
            .try_into()
            .map_err(|_| SnowflakeError::MalformedHeaders)?;

        reqwest::ClientBuilder::new()
            .user_agent("rsql-Snowflake-Driver")
            .default_headers(header_map)
            .build()
            .map_err(|_| SnowflakeError::ClientCreation.into())
    }

    async fn request_handle_partition(
        &mut self,
        handle: &str,
        partition: usize,
    ) -> Result<reqwest::Response> {
        let url = format!("{}/{handle}", self.base_url);

        self.client
            .lock()
            .await
            .get(&url)
            .query(&[("partition", partition.to_string())])
            .send()
            .await
            .map_err(|e| {
                error!("snowflake get request error: {:?}", e);
                SnowflakeError::Request.into()
            })
    }

    async fn request(&mut self, sql: &str) -> Result<reqwest::Response> {
        if self.jwt_expires_at < chrono::Utc::now() {
            let mut client = self.client.lock().await;
            *client = Self::new_client(&self.issuer, &self.subject, &self.key_pair)?;
        }

        self.client
            .lock()
            .await
            .post(&self.base_url)
            .body(
                json!({
                    "statement": sql,
                    "timeout": 10,
                    "parameters": {
                        "DATE_OUTPUT_FORMAT": DATE_FORMATS.0,
                        "TIME_OUTPUT_FORMAT": TIME_FORMATS.0,
                        "TIMESTAMP_LTZ_OUTPUT_FORMAT": DATETIME_NO_TZ_FORMATS.0,
                        "TIMESTAMP_NTZ_OUTPUT_FORMAT": DATETIME_NO_TZ_FORMATS.0,
                        "TIMESTAMP_OUTPUT_FORMAT": DATETIME_NO_TZ_FORMATS.0,
                        "TIMESTAMP_TZ_OUTPUT_FORMAT": DATETIME_TZ_FORMATS.0,
                    }
                })
                .to_string(),
            )
            .send()
            .await
            .map_err(|e| {
                error!("snowflake request error: {:?}", e);
                SnowflakeError::Request.into()
            })
    }

    fn parse_result_data(
        result_data: &serde_json::Value,
        column_definitions: &Vec<ColumnDefinition>,
    ) -> Result<Vec<Row>> {
        result_data["data"]
            .as_array()
            .ok_or(SnowflakeError::Response)?
            .iter()
            .map(|row| {
                row.as_array()
                    .ok_or(SnowflakeError::Response)?
                    .iter()
                    .zip(column_definitions.iter())
                    .map(|(value, column)| {
                        column.map_value(value).map_err(|e| {
                            error!("error: {:?}", e);
                            SnowflakeError::Response.into()
                        })
                    })
                    .collect::<Result<Vec<_>>>()
                    .map(Row::new)
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[async_trait]
impl crate::Connection for SnowflakeConnection {
    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let response = self
            .request(sql)
            .await?
            .error_for_status()
            .map_err(|_| SnowflakeError::Response)?;
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            error!("error: {:?}", e);
            SnowflakeError::Response
        })?;
        let row_count = response_json["data"][0][0]
            .as_str()
            .ok_or(SnowflakeError::Response)?
            .parse::<u64>()
            .map_err(|_| SnowflakeError::Response)?;
        Ok(row_count)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        Ok(Metadata::default())
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let response = self.request(sql).await?;
        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            error!("error: {:?}", e);
            SnowflakeError::Response
        })?;

        let handle = response_json["statementHandle"]
            .as_str()
            .ok_or(SnowflakeError::Response)?;
        let partitions = response_json["resultSetMetaData"]["partitionInfo"]
            .as_array()
            .ok_or(SnowflakeError::Response)?;
        let column_definitions: Vec<_> = response_json["resultSetMetaData"]["rowType"]
            .as_array()
            .ok_or(SnowflakeError::Response)?
            .iter()
            .map(ColumnDefinition::try_from_value)
            .collect::<Result<Vec<_>>>()?;

        let column_names: Vec<_> = column_definitions
            .iter()
            .map(|value| value.name.clone())
            .collect();

        let mut rows = Self::parse_result_data(&response_json, &column_definitions)?;
        if partitions.len() > 1 {
            for i in 1..partitions.len() {
                let response = self.request_handle_partition(handle, i).await?;
                let response_json: serde_json::Value = response.json().await.map_err(|e| {
                    error!("error: {:?}", e);
                    SnowflakeError::Response
                })?;
                rows.extend(Self::parse_result_data(
                    &response_json,
                    &column_definitions,
                )?);
            }
        }

        let qr = MemoryQueryResult::new(column_names, rows);
        Ok(Box::new(qr))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
struct ColumnDefinition {
    pub name: String,
    snowflake_type: String,
    scale: Option<u64>,
}

impl ColumnDefinition {
    fn new(name: String, snowflake_type: String, scale: Option<u64>) -> Self {
        Self {
            name,
            snowflake_type,
            scale,
        }
    }

    fn map_value(&self, value: &serde_json::Value) -> Result<Value> {
        if let serde_json::Value::Null = value {
            return Ok(Value::Null);
        }
        let value = value
            .as_str()
            .ok_or(SnowflakeError::ResponseContent("bad value".into()))?;
        Ok(match self.snowflake_type.to_lowercase().as_str() {
            "fixed" => {
                if self.scale.is_some() && self.scale.unwrap_or(0) > 0 {
                    Value::F64(
                        value
                            .parse()
                            .map_err(|_| SnowflakeError::ResponseContent("bad f64".into()))?,
                    )
                } else {
                    Value::I64(
                        value
                            .parse()
                            .map_err(|_| SnowflakeError::ResponseContent("bad i64".into()))?,
                    )
                }
            }
            "boolean" => Value::Bool(
                value
                    .parse()
                    .map_err(|_| SnowflakeError::ResponseContent("bad bool".into()))?,
            ),
            "date" => Value::Date(
                NaiveDate::parse_from_str(value, DATE_FORMATS.1)
                    .map_err(|_| SnowflakeError::ResponseContent("bad date".into()))?,
            ),
            "time" => Value::Time(
                NaiveTime::parse_from_str(value, TIME_FORMATS.1)
                    .map_err(|_| SnowflakeError::ResponseContent("bad time".into()))?,
            ),
            "timestamp_ntz" | "timestamp_ltz" => Value::DateTime(
                NaiveDateTime::parse_from_str(value, DATETIME_NO_TZ_FORMATS.1)
                    .map_err(|_| SnowflakeError::ResponseContent("bad datetime ntz".into()))?,
            ),
            "timestamp_tz" => Value::DateTime(
                NaiveDateTime::parse_from_str(value, DATETIME_TZ_FORMATS.1)
                    .map_err(|_| SnowflakeError::ResponseContent("bad datetime tz".into()))?,
            ),
            // includes "text" field for VARCHARs
            _ => Value::String(value.to_string()),
        })
    }

    fn try_from_value(value: &serde_json::Value) -> Result<Self> {
        let name = value["name"]
            .as_str()
            .ok_or(SnowflakeError::Response)?
            .to_string();
        let snowflake_type = value["type"]
            .as_str()
            .ok_or(SnowflakeError::Response)?
            .to_string();
        let scale = value["scale"].as_u64();

        Ok(Self::new(name, snowflake_type, scale))
    }
}
