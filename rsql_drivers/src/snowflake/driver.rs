use crate::connection::Row;
use crate::{snowflake::SnowflakeError, MemoryQueryResult, Metadata, QueryResult, Result, Value};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use jwt_simple::prelude::{Claims, Duration, RS256KeyPair, RS256PublicKey, RSAKeyPairLike};
use reqwest::header::HeaderMap;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlparser::dialect::{Dialect, SnowflakeDialect};
use std::collections::HashMap;
use std::fmt::Display;
use tokio::sync::Mutex;
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
        password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        Ok(Box::new(SnowflakeConnection::new(url, password)?))
    }
}

#[derive(Debug)]
pub(crate) struct SnowflakeConnection {
    url: String,
    base_url: String,
    issuer: Option<String>,
    subject: Option<String>,
    key_pair: Option<RS256KeyPair>,
    jwt_expires_at: Option<DateTime<Utc>>,
    client: Mutex<reqwest::Client>,
}

impl SnowflakeConnection {
    /// Establish a new connection to Snowflake
    ///
    /// # Errors
    /// Errors if the URL is malformed, the account, user, private key, or public key are missing,
    /// or if the private key or public key files are missing or malformed
    pub(crate) fn new(url: String, password: Option<String>) -> Result<SnowflakeConnection> {
        let parsed_url = Url::parse(url.as_str())?;
        let query_params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let base_url = parsed_url
            .host_str()
            .ok_or(SnowflakeError::MissingAccount)?
            .to_string();
        let account = base_url
            .split('.')
            .next()
            .ok_or(SnowflakeError::MissingAccount)?;
        let user = parsed_url.username();
        let base_url = format!("https://{base_url}/api/v2/statements");

        if let Some(password) = password {
            let client = Mutex::new(Self::new_client_oauth(&password)?);
            Ok(Self {
                url,
                base_url,
                issuer: None,
                subject: None,
                key_pair: None,
                jwt_expires_at: None,
                client,
            })
        } else {
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
            let key_pair = RS256KeyPair::from_pem(&private_key)
                .map_err(|_| SnowflakeError::MissingPrivateKey)?;

            let (issuer, subject) = get_issuer_and_subject(&public_key, account, user)?;
            let jwt_expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

            let client = Mutex::new(Self::new_client_keypair(&issuer, &subject, &key_pair)?);
            Ok(Self {
                url,
                base_url,
                issuer: Some(issuer),
                subject: Some(subject),
                key_pair: Some(key_pair),
                jwt_expires_at: Some(jwt_expires_at),
                client,
            })
        }
    }

    /// Create a new client
    ///
    /// # Errors
    /// Errors if there is an issue building the underlying client
    fn new_client(auth_token: &str, token_type: &str) -> Result<reqwest::Client> {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_owned(), format!("Bearer {auth_token}"));
        headers.insert("Content-Type".to_owned(), "application/json".to_owned());
        headers.insert(
            "X-Snowflake-Authorization-Token-Type".to_owned(),
            token_type.to_owned(),
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

    /// Create a new client using an OAuth token
    ///
    /// # Errors
    /// Errors if there is an issue building client headers
    fn new_client_oauth(oauth_token: &str) -> Result<reqwest::Client> {
        Self::new_client(oauth_token, "OAUTH")
    }

    /// Create a new client using a keypair
    ///
    /// # Errors
    /// Errors if there is an issue signing the JWT
    fn new_client_keypair(
        issuer: &str,
        subject: &str,
        key_pair: &RS256KeyPair,
    ) -> Result<reqwest::Client> {
        let claims = Claims::create(Duration::from_hours(1))
            .with_issuer(issuer)
            .with_subject(subject);

        let token = key_pair
            .sign(claims)
            .map_err(|_| SnowflakeError::JwtSignature)?;

        Self::new_client(&token, "KEYPAIR_JWT")
    }

    /// Request a subsequent data partition for a given statement handle
    ///
    /// # Errors
    /// Errors if the request to Snowflake fails
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
            .map_err(|e| SnowflakeError::Request(e).into())
    }

    /// If this connection is configured to use a key pair, check if the JWT has expired and refresh it if necessary
    ///
    /// # Errors
    /// Errors if creating a new client with the key pair fails
    async fn check_jwt_refresh(&mut self) -> Result<()> {
        if let (Some(jwt_expires_at), Some(issuer), Some(subject), Some(key_pair)) = (
            &self.jwt_expires_at,
            &self.issuer,
            &self.subject,
            &self.key_pair,
        ) {
            if *jwt_expires_at < chrono::Utc::now() {
                let new_client = Self::new_client_keypair(issuer, subject, key_pair)?;
                let mut client = self.client.lock().await;
                *client = new_client;
            }
        }
        Ok(())
    }

    /// Execute a SQL query against the Snowflake API.
    ///
    /// # Errors
    /// Errors if the request fails to receive a response
    async fn request(&mut self, sql: &str) -> Result<reqwest::Response> {
        self.check_jwt_refresh().await?;
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
            .map_err(|e| SnowflakeError::Request(e).into())
    }

    /// Parse row data from snowflake response
    ///
    /// # Errors
    /// Errors if the `result_data["data"]` is not an array or if the row data is not an array
    fn parse_result_data(
        result_data: &serde_json::Value,
        column_definitions: &[ColumnDefinition],
    ) -> Result<Vec<Row>> {
        result_data["data"]
            .as_array()
            .ok_or(SnowflakeError::ResponseContent(
                "Snowflake Response missing row data".into(),
            ))?
            .iter()
            .map(|row| {
                row.as_array()
                    .ok_or(SnowflakeError::ResponseContent(
                        "row data is not an array".into(),
                    ))?
                    .iter()
                    .zip(column_definitions.iter())
                    .map(|(value, column)| column.convert_to_value(value))
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()
    }

    #[cfg(test)]
    fn set_base_url(&mut self, base_url: &str) {
        self.base_url = format!("{base_url}/api/v2/statements");
    }
}

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

/// Get issuer and subject for a public key
/// Snowflake requires the issuer and subject to be constructed with the account, user, and fingerprint
///
/// # Errors
/// Errors if the public key is malformed
fn get_issuer_and_subject(public_key: &str, account: &str, user: &str) -> Result<(String, String)> {
    let fingerprint = public_key_fingerprint(public_key)?;
    let issuer = format!("{account}.{user}.SHA256:{fingerprint}");
    let subject = format!("{account}.{user}");
    Ok((issuer, subject))
}

#[async_trait]
impl crate::Connection for SnowflakeConnection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let response = self
            .request(sql)
            .await?
            .error_for_status()
            .map_err(SnowflakeError::Response)?;
        let response_json: serde_json::Value =
            response.json().await.map_err(SnowflakeError::Response)?;
        let row_count = response_json["data"][0][0]
            .as_str()
            .ok_or(SnowflakeError::ResponseContent(
                "Query executed: row count not found".into(),
            ))?
            .parse::<u64>()
            .map_err(|e| {
                SnowflakeError::ResponseContent(format!(
                    "Query executed: row count not a number: {e}"
                ))
            })?;
        Ok(row_count)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        Ok(Metadata::default())
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let response = self
            .request(sql)
            .await?
            .error_for_status()
            .map_err(SnowflakeError::Response)?;
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| SnowflakeError::ResponseContent(format!("Error parsing Response: {e}")))?;

        let handle =
            response_json["statementHandle"]
                .as_str()
                .ok_or(SnowflakeError::ResponseContent(
                    "No handle in Response".into(),
                ))?;
        let partitions = response_json["resultSetMetaData"]["partitionInfo"]
            .as_array()
            .ok_or(SnowflakeError::ResponseContent(
                "No partition data in response".into(),
            ))?;
        let column_definitions: Vec<_> = response_json["resultSetMetaData"]["rowType"]
            .as_array()
            .ok_or(SnowflakeError::ResponseContent(
                "No ResultSet row type info in response".into(),
            ))?
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
                let response = self
                    .request_handle_partition(handle, i)
                    .await?
                    .error_for_status()
                    .map_err(SnowflakeError::Response)?;
                let response_json: serde_json::Value = response.json().await.map_err(|e| {
                    SnowflakeError::ResponseContent(format!(
                        "Error parsing partition response: {e}"
                    ))
                })?;
                rows.extend(Self::parse_result_data(
                    &response_json,
                    &column_definitions,
                )?);
            }
        }

        Ok(Box::new(MemoryQueryResult::new(column_names, rows)))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(SnowflakeDialect {})
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

    fn translate_error(value: &str, error: impl Display) -> SnowflakeError {
        SnowflakeError::ResponseContent(format!("could not parse {value}: {error}"))
    }

    fn convert_to_value(&self, value: &serde_json::Value) -> Result<Value> {
        if let serde_json::Value::Null = value {
            return Ok(Value::Null);
        }
        let value = value
            .as_str()
            .ok_or(SnowflakeError::ResponseContent(format!(
                "row data contained non-string value before parsing {value}"
            )))?;
        Ok(match self.snowflake_type.to_lowercase().as_str() {
            "fixed" => {
                if self.scale.is_some() && self.scale.unwrap_or(0) > 0 {
                    Value::F64(value.parse().map_err(|e| Self::translate_error(value, e))?)
                } else {
                    Value::I64(value.parse().map_err(|e| Self::translate_error(value, e))?)
                }
            }
            "boolean" => Value::Bool(value.parse().map_err(|e| Self::translate_error(value, e))?),
            "date" => Value::Date(
                NaiveDate::parse_from_str(value, DATE_FORMATS.1)
                    .map_err(|e| Self::translate_error(value, e))?,
            ),
            "time" => Value::Time(
                NaiveTime::parse_from_str(value, TIME_FORMATS.1)
                    .map_err(|e| Self::translate_error(value, e))?,
            ),
            "timestamp_ntz" | "timestamp_ltz" => Value::DateTime(
                NaiveDateTime::parse_from_str(value, DATETIME_NO_TZ_FORMATS.1)
                    .map_err(|e| Self::translate_error(value, e))?,
            ),
            "timestamp_tz" => Value::DateTime(
                NaiveDateTime::parse_from_str(value, DATETIME_TZ_FORMATS.1)
                    .map_err(|e| Self::translate_error(value, e))?,
            ),
            // includes "text" field for VARCHARs
            _ => Value::String(value.to_string()),
        })
    }

    fn try_from_value(value: &serde_json::Value) -> Result<Self> {
        let name = value["name"]
            .as_str()
            .ok_or(SnowflakeError::ResponseContent(
                "missing column name in response".into(),
            ))?
            .to_string();
        let snowflake_type = value["type"]
            .as_str()
            .ok_or(SnowflakeError::ResponseContent(
                "missing column type in response".into(),
            ))?
            .to_string();
        let scale = value["scale"].as_u64();

        Ok(Self::new(name, snowflake_type, scale))
    }
}

#[cfg(test)]
mod test {
    use crate::Connection;

    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[allow(clippy::too_many_lines)]
    fn initial_response_json() -> serde_json::Value {
        json!({
            "resultSetMetaData": {
                "numRows": 2,
                "format": "jsonv2",
                "partitionInfo": [
                    {
                        "rowCount": 1,
                        "uncompressedSize": 133
                    },
                    {
                        "rowCount": 1,
                        "uncompressedSize": 133
                    }
                ],
                "rowType": [
                    {
                        "name": "Int",
                        "database": "",
                        "schema": "",
                        "table": "",
                        "nullable": false,
                        "type": "fixed",
                        "byteLength": null,
                        "length": null,
                        "scale": 0,
                        "precision": 1,
                        "collation": null
                    },
                    {
                        "name": "Float",
                        "database": "",
                        "schema": "",
                        "table": "",
                        "nullable": false,
                        "type": "fixed",
                        "byteLength": null,
                        "length": null,
                        "scale": 1,
                        "precision": 2,
                        "collation": null
                    },
                    {
                        "name": "Boolean",
                        "database": "",
                        "schema": "",
                        "table": "",
                        "nullable": true,
                        "type": "boolean",
                        "byteLength": null,
                        "length": null,
                        "scale": null,
                        "precision": null,
                        "collation": null
                    },
                    {
                        "name": "Time",
                        "database": "",
                        "schema": "",
                        "table": "",
                        "nullable": true,
                        "type": "time",
                        "byteLength": null,
                        "length": null,
                        "scale": 9,
                        "precision": 0,
                        "collation": null
                    },
                    {
                        "name": "Date",
                        "database": "",
                        "schema": "",
                        "table": "",
                        "nullable": true,
                        "type": "date",
                        "byteLength": null,
                        "length": null,
                        "scale": null,
                        "precision": null,
                        "collation": null
                    },
                    {
                        "name": "DateTimeNTZ",
                        "database": "",
                        "schema": "",
                        "table": "",
                        "nullable": true,
                        "type": "timestamp_ntz",
                        "byteLength": null,
                        "length": null,
                        "scale": 9,
                        "precision": 0,
                        "collation": null
                    },
                    {
                        "name": "DateTimeTZ",
                        "database": "",
                        "schema": "",
                        "table": "",
                        "nullable": true,
                        "type": "timestamp_tz",
                        "byteLength": null,
                        "length": null,
                        "scale": 9,
                        "precision": 0,
                        "collation": null
                    }
                ]
            },
            "data": [
                [
                    "1",
                    "2.1",
                    "false",
                    "19:57:48.000000000",
                    "2024-08-14",
                    "2024-08-14T19:57:48.000000000",
                    "2024-08-14T19:57:48.000000000-0700"
                ],
            ],
            "code": "090001",
            "statementStatusUrl": "/api/v2/statements/01b69c52-0002-cff6-007b-7807000435b2?requestId=d6b5ab52-ffdb-41ec-84f1-33709b075eaf",
            "requestId": "d6b5ab52-ffdb-41ec-84f1-33709b075eaf",
            "sqlState": "00000",
            "statementHandle": "01b69c52-0002-cff6-007b-7807000435b2",
            "message": "Statement executed successfully.",
            "createdOn": 123_456
        })
    }

    fn partition_handle_response_json() -> serde_json::Value {
        json!({
            "data": [
                [
                    "2",
                    "3.1",
                    "true",
                    "23:59:59.123456789",
                    "2000-01-01",
                    "2000-01-01T23:59:59.123456789",
                    "2000-01-01T23:59:59.123456789-0000"
                ]
            ]
        })
    }

    #[tokio::test]
    async fn test_query_against_mock() -> anyhow::Result<()> {
        let mock = MockServer::start().await;
        let response_json = initial_response_json();
        let handle = "01b69c52-0002-cff6-007b-7807000435b2";
        Mock::given(method("POST"))
            .and(path("/api/v2/statements"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_json))
            .mount(&mock)
            .await;

        Mock::given(method("GET"))
            .and(path(format!("/api/v2/statements/{handle}")))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(partition_handle_response_json()),
            )
            .mount(&mock)
            .await;

        let database_url = "snowflake://abc123.snowflakecomputing.com/?user=test".to_string();
        let mut connection =
            SnowflakeConnection::new(database_url.clone(), Some("auth_token".to_string()))?;
        assert_eq!(database_url, connection.url().as_str());
        connection.set_base_url(&mock.uri());

        let mut result = connection
            .query(
                "SELECT Int, Float, Boolean, Time, Date, DateTimeNTZ, DateTimeTZ, FROM table LIMIT 2",
            )
            .await?;
        assert_eq!(
            result.next().await,
            Some(vec![
                Value::I64(1),
                Value::F64(2.1),
                Value::Bool(false),
                Value::Time(NaiveTime::from_hms_nano_opt(19, 57, 48, 0).expect("invalid time")),
                Value::Date(NaiveDate::from_ymd_opt(2024, 8, 14).expect("invalid date")),
                Value::DateTime(NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2024, 8, 14).expect("invalid date"),
                    NaiveTime::from_hms_opt(19, 57, 48).expect("invalid time")
                )),
                Value::DateTime(
                    DateTime::parse_from_rfc3339("2024-08-14T19:57:48.000000000+00:00")
                        .expect("invalid datetime")
                        .naive_utc()
                )
            ])
        );
        assert_eq!(
            result.next().await,
            Some(vec![
                Value::I64(2),
                Value::F64(3.1),
                Value::Bool(true),
                Value::Time(
                    NaiveTime::from_hms_nano_opt(23, 59, 59, 123_456_789).expect("invalid time")
                ),
                Value::Date(NaiveDate::from_ymd_opt(2000, 1, 1).expect("invalid date")),
                Value::DateTime(NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2000, 1, 1).expect("invalid date"),
                    NaiveTime::from_hms_nano_opt(23, 59, 59, 123_456_789).expect("invalid time")
                )),
                Value::DateTime(
                    DateTime::parse_from_rfc3339("2000-01-01T23:59:59.123456789-00:00")
                        .expect("invalid datetime")
                        .naive_utc()
                )
            ])
        );
        Ok(())
    }

    #[test]
    fn test_get_issuer_and_subject() {
        let keypair = RS256KeyPair::generate(2048).expect("cannot generate key for tests");
        let public_cert = keypair
            .public_key()
            .to_pem()
            .expect("cannot generate cert for tests");
        let expected_thumbprint =
            public_key_fingerprint(&public_cert).expect("cannot generate thumbprint");
        let (issuer, subject) = get_issuer_and_subject(&public_cert, "abc123", "test")
            .expect("Failed to get issuer and subject");
        assert_eq!(subject, format!("abc123.test"));
        assert_eq!(issuer, format!("abc123.test.SHA256:{expected_thumbprint}"));
    }

    #[test]
    fn test_column_maps_null() {
        let column = ColumnDefinition::new("column".to_string(), "fixed".to_string(), None);
        assert_eq!(
            column.convert_to_value(&json!(null)).ok(),
            Some(Value::Null)
        );
    }

    #[test]
    fn test_float_column() {
        let column = ColumnDefinition::new("float".to_string(), "fixed".to_string(), Some(5));
        assert_eq!(
            column
                .convert_to_value(&json!("1.23456"))
                .expect("failed to convert to value"),
            Value::F64(1.23456)
        );
        assert_eq!(
            column
                .convert_to_value(&json!("0.00001"))
                .expect("failed to convert to value"),
            Value::F64(0.00001)
        );
        assert_eq!(
            column
                .convert_to_value(&json!("-123.45678"))
                .expect("failed to convert to value"),
            Value::F64(-123.45678)
        );
        assert_eq!(
            column
                .convert_to_value(&json!("9999999.99999"))
                .expect("failed to convert to value"),
            Value::F64(9_999_999.999_99)
        );
        assert!(column.convert_to_value(&json!("not_a_number")).is_err());
        assert!(column.convert_to_value(&json!("1,23456")).is_err());
    }

    #[test]
    fn test_int_column() {
        let int_column = ColumnDefinition::new("int".to_string(), "fixed".to_string(), None);

        assert_eq!(
            int_column
                .convert_to_value(&json!("1"))
                .expect("failed to convert to _value"),
            Value::I64(1)
        );
        assert_eq!(
            int_column
                .convert_to_value(&json!("0"))
                .expect("failed to convert to _value"),
            Value::I64(0)
        );
        assert_eq!(
            int_column
                .convert_to_value(&json!("156516516514"))
                .expect("invalid value"),
            Value::I64(156_516_516_514)
        );
        assert_eq!(
            int_column
                .convert_to_value(&json!("-9999999"))
                .expect("failed to convert to value"),
            Value::I64(-9_999_999)
        );
        assert_eq!(
            int_column
                .convert_to_value(&json!(i64::MIN.to_string()))
                .expect("failed to convert to value"),
            Value::I64(i64::MIN)
        );
        assert_eq!(
            int_column
                .convert_to_value(&json!(i64::MAX.to_string()))
                .expect("failed to convert to value"),
            Value::I64(i64::MAX)
        );
        assert!(int_column.convert_to_value(&json!("1.3434")).is_err());
    }

    #[test]
    fn test_boolean_column() {
        let column = ColumnDefinition::new("bool".to_string(), "boolean".to_string(), None);
        assert_eq!(
            column
                .convert_to_value(&json!("true"))
                .expect("failed to convert to value"),
            Value::Bool(true)
        );
        assert_eq!(
            column
                .convert_to_value(&json!("false"))
                .expect("failed to convert to value"),
            Value::Bool(false)
        );
        assert!(column.convert_to_value(&json!("not_a_boolean")).is_err());
    }

    #[test]
    fn test_date_column() {
        let column = ColumnDefinition::new("date".to_string(), "date".to_string(), None);
        assert_eq!(
            column
                .convert_to_value(&json!("2024-08-24"))
                .expect("failed to convert to value"),
            Value::Date(NaiveDate::from_ymd_opt(2024, 8, 24).expect("invalid date"))
        );
        assert_eq!(
            column
                .convert_to_value(&json!("1993-05-07"))
                .expect("failed to convert to value"),
            Value::Date(NaiveDate::from_ymd_opt(1993, 5, 7).expect("invalid date"))
        );
        assert!(column.convert_to_value(&json!("not_a_date")).is_err());
    }

    #[test]
    fn test_time_column() {
        let column = ColumnDefinition::new("time".to_string(), "time".to_string(), None);
        assert_eq!(
            column
                .convert_to_value(&json!("12:00:00.000000000"))
                .expect("failed to convert to value"),
            Value::Time(NaiveTime::from_hms_opt(12, 0, 0).expect("invalid time"))
        );
        assert_eq!(
            column
                .convert_to_value(&json!("00:00:00.123456789"))
                .expect("failed to convert to value"),
            Value::Time(NaiveTime::from_hms_nano_opt(0, 0, 0, 123_456_789).expect("invalid time"))
        );
        assert!(column.convert_to_value(&json!("not_a_time")).is_err());
    }

    #[test]
    fn test_datetime_ntz_column() {
        let column = ColumnDefinition::new(
            "timestamp_ntz".to_string(),
            "timestamp_ntz".to_string(),
            None,
        );
        assert_eq!(
            column
                .convert_to_value(&json!("2025-01-01T23:59:59.000000000"))
                .expect("failed to convert to value"),
            Value::DateTime(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid date"),
                NaiveTime::from_hms_nano_opt(23, 59, 59, 0).expect("invalid time")
            ))
        );
        assert_eq!(
            column
                .convert_to_value(&json!("2001-01-01T23:59:59.123456789"))
                .expect("failed to convert to value"),
            Value::DateTime(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2001, 1, 1).expect("invalid date"),
                NaiveTime::from_hms_nano_opt(23, 59, 59, 123_456_789).expect("invalid time")
            ))
        );
        assert!(column.convert_to_value(&json!("not_a_datetime")).is_err());
    }

    #[test]
    fn test_datetime_ltz_column() {
        let column = ColumnDefinition::new(
            "timestamp_ltz".to_string(),
            "timestamp_ltz".to_string(),
            None,
        );
        assert_eq!(
            column
                .convert_to_value(&json!("2025-01-01T23:59:59.000000000"))
                .expect("failed to convert to value"),
            Value::DateTime(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid date"),
                NaiveTime::from_hms_nano_opt(23, 59, 59, 0).expect("invalid time")
            ))
        );
        assert_eq!(
            column
                .convert_to_value(&json!("2001-01-01T23:59:59.123456789"))
                .expect("failed to convert to value"),
            Value::DateTime(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2001, 1, 1).expect("invalid date"),
                NaiveTime::from_hms_nano_opt(23, 59, 59, 123_456_789).expect("invalid value")
            ))
        );
        assert!(column.convert_to_value(&json!("not_a_datetime")).is_err());
    }

    #[test]
    fn test_datetime_tz_column() {
        let column =
            ColumnDefinition::new("timestamp_tz".to_string(), "timestamp_tz".to_string(), None);
        assert_eq!(
            column
                .convert_to_value(&json!("2025-01-01T23:59:59.000000000-0700"))
                .expect("failed to convert to value"),
            Value::DateTime(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2025, 1, 1).expect("invalid date"),
                NaiveTime::from_hms_nano_opt(23, 59, 59, 0).expect("invalid time")
            ))
        );
        assert_eq!(
            column
                .convert_to_value(&json!("2001-01-01T23:59:59.123456789+0000"))
                .expect("failed to convert to value"),
            Value::DateTime(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2001, 1, 1).expect("invalid date"),
                NaiveTime::from_hms_nano_opt(23, 59, 59, 123_456_789).expect("invalid time")
            ))
        );
        assert!(column.convert_to_value(&json!("not_a_datetime")).is_err());
    }
}
