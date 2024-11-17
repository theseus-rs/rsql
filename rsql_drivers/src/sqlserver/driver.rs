use crate::error::Result;
use crate::sqlserver::metadata;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, Metadata, QueryResult};
use async_trait::async_trait;
use futures_util::stream::TryStreamExt;
use sqlparser::dialect::{Dialect, MsSqlDialect};
use std::collections::HashMap;
use std::string::ToString;
use tiberius::{AuthMethod, Client, Column, Config, EncryptionLevel, QueryItem, Row};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "sqlserver"
    }

    async fn connect(
        &self,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let connection = Connection::new(url, password).await?;
        Ok(Box::new(connection))
    }
}

#[derive(Debug)]
pub(crate) struct Connection {
    url: String,
    client: Client<Compat<TcpStream>>,
}

impl Connection {
    pub(crate) async fn new(url: String, password: Option<String>) -> Result<Connection> {
        let parsed_url = url::Url::parse(url.as_str())?;
        let mut params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let trust_server_certificate = params
            .remove("TrustServerCertificate")
            .map_or(false, |value| value == "true");
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

        let client = Client::connect(config, tcp.compat_write()).await?;
        let connection = Connection { url, client };

        Ok(connection)
    }
}

#[async_trait]
impl crate::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let result = self.client.execute(sql, &[]).await?;
        let rows = result.rows_affected()[0];
        Ok(rows)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let mut query_stream = self.client.query(sql, &[]).await?;
        let mut columns: Vec<String> = Vec::new();

        let mut rows = Vec::new();
        while let Some(item) = query_stream.try_next().await? {
            if let QueryItem::Metadata(meta) = item {
                if meta.result_index() == 0 {
                    for column in meta.columns() {
                        columns.push(column.name().to_string());
                    }
                }
            } else if let QueryItem::Row(row) = item {
                let mut row_data = Vec::new();
                if row.result_index() == 0 {
                    for (index, column) in row.columns().iter().enumerate() {
                        let value = convert_to_value(&row, column, index)?;
                        row_data.push(value);
                    }
                }
                rows.push(row_data);
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(MsSqlDialect {})
    }
}

#[expect(clippy::same_functions_in_if_condition)]
fn convert_to_value(row: &Row, column: &Column, index: usize) -> Result<Value> {
    let column_name = column.name();

    if let Ok(value) = row.try_get(index) {
        let value: Option<&str> = value;
        match value {
            Some(v) => Ok(Value::String(v.to_string())),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<&[u8]> = value;
        match value {
            Some(v) => Ok(Value::Bytes(v.to_vec())),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<u8> = value;
        match value {
            Some(v) => Ok(Value::U8(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i16> = value;
        match value {
            Some(v) => Ok(Value::I16(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i32> = value;
        match value {
            Some(v) => Ok(Value::I32(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i64> = value;
        match value {
            Some(v) => Ok(Value::I64(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<f32> = value;
        match value {
            Some(v) => Ok(Value::F32(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<f64> = value;
        match value {
            Some(v) => Ok(Value::F64(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<bool> = value;
        match value {
            Some(v) => Ok(Value::Bool(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<rust_decimal::Decimal> = value;
        match value {
            Some(v) => Ok(Value::String(v.to_string())),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDate> = value;
        match value {
            Some(v) => Ok(Value::Date(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveTime> = value;
        match value {
            Some(v) => Ok(Value::Time(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDateTime> = value;
        match value {
            Some(v) => Ok(Value::DateTime(v)),
            None => Ok(Value::Null),
        }
    } else {
        let column_type = format!("{:?}", column.column_type());
        let type_name = format!("{column_type:?}");
        return Err(UnsupportedColumnType {
            column_name: column_name.to_string(),
            column_type: type_name,
        });
    }
}
