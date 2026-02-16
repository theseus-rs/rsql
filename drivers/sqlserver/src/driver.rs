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
        let mut params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let trust_server_certificate = params
            .remove("TrustServerCertificate")
            .is_some_and(|value| value == "true");
        let encryption = params
            .remove("encryption")
            .unwrap_or("required".to_string());

        let host = parsed_url.host_str().unwrap_or("localhost");
        let port = parsed_url.port().unwrap_or(1433);
        let database = parsed_url.path().replace('/', "");
        let username = parsed_url.username();

        let mut config = Config::new();
        config.host(host);
        config.port(port);

        if !database.is_empty() {
            config.database(database);
        }

        if !username.is_empty() {
            config.authentication(AuthMethod::sql_server(
                username,
                password.expect("password is required"),
            ));
        }

        if trust_server_certificate {
            config.trust_cert();
        }

        match encryption.as_str() {
            "off" => config.encryption(EncryptionLevel::Off),
            "on" => config.encryption(EncryptionLevel::On),
            "not_supported" => config.encryption(EncryptionLevel::NotSupported),
            _ => config.encryption(EncryptionLevel::Required),
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
