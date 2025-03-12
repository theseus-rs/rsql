use crate::metadata;
use async_trait::async_trait;
use chrono::{Datelike, Timelike};
use file_type::FileType;
use futures_util::stream::TryStreamExt;
use rsql_driver::Error::{InvalidUrl, IoError, UnsupportedColumnType};
use rsql_driver::{MemoryQueryResult, Metadata, QueryResult, Result, Value};
use sqlparser::dialect::{Dialect, MsSqlDialect};
use std::collections::HashMap;
use std::string::ToString;
use tiberius::{AuthMethod, Client, Column, Config, EncryptionLevel, QueryItem, Row};
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

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let result = self
            .client
            .execute(sql, &[])
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let rows = result.rows_affected()[0];
        Ok(rows)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let mut query_stream = self
            .client
            .query(sql, &[])
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let mut columns: Vec<String> = Vec::new();

        let mut rows = Vec::new();
        while let Some(item) = query_stream
            .try_next()
            .await
            .map_err(|error| IoError(error.to_string()))?
        {
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
#[expect(clippy::too_many_lines)]
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
            Some(v) => Ok(Value::Decimal(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDate> = value;
        match value {
            Some(v) => {
                let year = i16::try_from(v.year())?;
                let month = i8::try_from(v.month())?;
                let day = i8::try_from(v.day())?;
                let date = jiff::civil::date(year, month, day);
                Ok(Value::Date(date))
            }
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveTime> = value;
        match value {
            Some(v) => {
                let hour = i8::try_from(v.hour())?;
                let minute = i8::try_from(v.minute())?;
                let second = i8::try_from(v.second())?;
                let nanosecond = i32::try_from(v.nanosecond())?;
                let time = jiff::civil::time(hour, minute, second, nanosecond);
                Ok(Value::Time(time))
            }
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDateTime> = value;
        match value {
            Some(v) => {
                let year = i16::try_from(v.year())?;
                let month = i8::try_from(v.month())?;
                let day = i8::try_from(v.day())?;
                let hour = i8::try_from(v.hour())?;
                let minute = i8::try_from(v.minute())?;
                let second = i8::try_from(v.second())?;
                let nanosecond = i32::try_from(v.nanosecond())?;
                let date_time =
                    jiff::civil::datetime(year, month, day, hour, minute, second, nanosecond);
                Ok(Value::DateTime(date_time))
            }
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
