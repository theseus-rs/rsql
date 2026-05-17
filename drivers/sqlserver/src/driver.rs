use crate::metadata;
use crate::results::SqlServerQueryResult;
use async_trait::async_trait;
use file_type::FileType;
use futures_util::stream::TryStreamExt;
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{Metadata, QueryResult, Result, ToSql, Value, convert_to_at_placeholders};
use sqlparser::dialect::{Dialect, MsSqlDialect};
use std::collections::HashMap;
use std::string::ToString;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, Query, QueryItem, Row};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "sqlserver"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url).map_err(|error| InvalidUrl(error.to_string()))?;
        let password = parsed_url.password().map(ToString::to_string);
        let connection = Connection::new(url, password).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Connection {
    url: String,
    client: Client<Compat<TcpStream>>,
}

impl Connection {
    pub(crate) async fn new(url: &str, password: Option<String>) -> Result<Connection> {
        let parsed_url = Url::parse(url)?;
        let mut params: HashMap<String, String> = parsed_url
            .query_pairs()
            .map(|(key, value)| (key.to_ascii_lowercase(), value.into_owned()))
            .collect();

        let mut host = parsed_url.host_str().unwrap_or("localhost").to_string();
        let mut port = parsed_url.port().unwrap_or(1433);
        let mut database = parsed_url.path().replace('/', "");
        let mut username = parsed_url.username().to_string();
        let mut password = password;

        if let Some(server) = take_first(&mut params, &["server"]) {
            let (server_host, server_port) = parse_server(&server);
            host = server_host;
            if let Some(server_port) = server_port {
                port = server_port;
            }
        }
        if let Some(database_param) = take_first(&mut params, &["database"]) {
            database = database_param;
        }
        if let Some(user_param) = take_first(&mut params, &["uid", "username", "user", "user id"]) {
            username = user_param;
        }
        if let Some(password_param) = take_first(&mut params, &["password", "pwd"]) {
            password = Some(password_param);
        }

        let integrated_security = take_first(&mut params, &["integratedsecurity"])
            .map(|value| parse_bool(&value, "IntegratedSecurity"))
            .transpose()?
            .unwrap_or(false);
        let trust_server_certificate = take_first(&mut params, &["trustservercertificate"])
            .map(|value| parse_bool(&value, "TrustServerCertificate"))
            .transpose()?
            .unwrap_or(false);
        let trust_server_certificate_ca = take_first(&mut params, &["trustservercertificateca"]);
        let encrypt = take_first(&mut params, &["encrypt"]);
        let legacy_encryption = take_first(&mut params, &["encryption"]);
        let application_name = take_first(&mut params, &["application name", "applicationname"]);

        if trust_server_certificate && trust_server_certificate_ca.is_some() {
            return Err(InvalidUrl(
                "TrustServerCertificate and TrustServerCertificateCA are mutually exclusive"
                    .to_string(),
            ));
        }

        let mut config = Config::new();
        config.host(&host);
        config.port(port);

        if !database.is_empty() {
            config.database(database);
        }

        if integrated_security {
            #[cfg(windows)]
            {
                config.authentication(AuthMethod::Integrated);
            }
            #[cfg(not(windows))]
            {
                return Err(InvalidUrl(
                    "IntegratedSecurity is only supported on Windows builds".to_string(),
                ));
            }
        } else if !username.is_empty() {
            config.authentication(AuthMethod::sql_server(
                &username,
                password.as_deref().unwrap_or(""),
            ));
        }

        if trust_server_certificate {
            config.trust_cert();
        } else if let Some(ca_path) = trust_server_certificate_ca {
            config.trust_cert_ca(ca_path);
        }

        let encryption_level = if let Some(value) = encrypt {
            parse_encrypt(&value)?
        } else if let Some(value) = legacy_encryption {
            parse_legacy_encryption(&value)?
        } else {
            EncryptionLevel::Required
        };
        config.encryption(encryption_level);

        if let Some(name) = application_name {
            config.application_name(name);
        }

        let tcp = TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        let client = Client::connect(config, tcp.compat_write())
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let connection = Connection {
            url: url.to_string(),
            client,
        };

        Ok(connection)
    }
}

fn take_first(params: &mut HashMap<String, String>, keys: &[&str]) -> Option<String> {
    let mut found = None;
    for key in keys {
        if let Some(value) = params.remove(*key)
            && found.is_none()
        {
            found = Some(value);
        }
    }
    found
}

fn parse_bool(value: &str, name: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" => Ok(true),
        "false" | "no" => Ok(false),
        other => Err(InvalidUrl(format!(
            "invalid value '{other}' for {name}; expected true, false, yes, or no"
        ))),
    }
}

fn parse_encrypt(value: &str) -> Result<EncryptionLevel> {
    if value.eq_ignore_ascii_case("DANGER_PLAINTEXT") {
        return Ok(EncryptionLevel::NotSupported);
    }
    Ok(if parse_bool(value, "encrypt")? {
        EncryptionLevel::Required
    } else {
        EncryptionLevel::Off
    })
}

fn parse_legacy_encryption(value: &str) -> Result<EncryptionLevel> {
    match value.to_ascii_lowercase().as_str() {
        "off" => Ok(EncryptionLevel::Off),
        "on" => Ok(EncryptionLevel::On),
        "not_supported" => Ok(EncryptionLevel::NotSupported),
        "required" => Ok(EncryptionLevel::Required),
        other => Err(InvalidUrl(format!(
            "invalid value '{other}' for encryption; expected off, on, not_supported, or required"
        ))),
    }
}

fn parse_server(server: &str) -> (String, Option<u16>) {
    let trimmed = server.strip_prefix("tcp:").unwrap_or(server);
    if let Some((host, port)) = trimmed.rsplit_once(',')
        && let Ok(port) = port.trim().parse::<u16>()
    {
        return (host.trim().to_string(), Some(port));
    }
    (trimmed.to_string(), None)
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let sql = convert_to_at_placeholders(sql);
        let values = rsql_driver::to_values(params);
        let mut query = Query::new(sql);
        for value in &values {
            bind_tiberius_value(&mut query, value);
        }
        let result = query
            .execute(&mut self.client)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let rows = result.rows_affected()[0];
        Ok(rows)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let sql = convert_to_at_placeholders(sql);
        let values = rsql_driver::to_values(params);
        let mut query = Query::new(sql);
        for value in &values {
            bind_tiberius_value(&mut query, value);
        }
        let mut query_stream = query
            .query(&mut self.client)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let mut columns: Vec<String> = Vec::new();

        let mut native_rows: Vec<Row> = Vec::new();
        while let Some(item) = query_stream
            .try_next()
            .await
            .map_err(|error| IoError(error.to_string()))?
        {
            match item {
                QueryItem::Metadata(meta) if meta.result_index() == 0 => {
                    for column in meta.columns() {
                        columns.push(column.name().to_string());
                    }
                }
                QueryItem::Row(row) if row.result_index() == 0 => {
                    native_rows.push(row);
                }
                _ => {}
            }
        }

        let query_result = SqlServerQueryResult::new(columns, native_rows);
        Ok(Box::new(query_result))
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(MsSqlDialect {})
    }
}

fn bind_tiberius_value<'a>(query: &mut Query<'a>, value: &'a Value) {
    match value {
        Value::Null => query.bind(Option::<String>::None),
        Value::Bool(v) => query.bind(*v),
        Value::I8(v) => query.bind(i16::from(*v)),
        Value::I16(v) => query.bind(*v),
        Value::I32(v) => query.bind(*v),
        Value::I64(v) => query.bind(*v),
        Value::U8(v) => query.bind(*v),
        Value::U16(v) => query.bind(i32::from(*v)),
        Value::U32(v) => query.bind(i64::from(*v)),
        Value::U64(v) => query.bind(*v as i64),
        Value::F32(v) => query.bind(*v),
        Value::F64(v) => query.bind(*v),
        Value::String(v) => query.bind(v.as_str()),
        Value::Bytes(v) => query.bind(v.as_slice()),
        Value::Decimal(v) => query.bind(v.to_string()),
        _ => query.bind(value.to_string()),
    }
}
