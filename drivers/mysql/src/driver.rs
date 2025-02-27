use crate::metadata;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use file_type::FileType;
use rsql_driver::Error::{InvalidUrl, IoError, UnsupportedColumnType};
use rsql_driver::{MemoryQueryResult, Metadata, QueryResult, Result, Value};
use sqlparser::dialect::{Dialect, MySqlDialect};
use sqlx::mysql::{MySqlColumn, MySqlConnectOptions, MySqlRow};
use sqlx::types::time::OffsetDateTime;
use sqlx::{Column, MySqlPool, Row};
use std::str::FromStr;
use std::string::ToString;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "mysql"
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
    pool: MySqlPool,
}

impl Connection {
    pub(crate) async fn new(url: &str, _password: Option<String>) -> Result<Connection> {
        let options =
            MySqlConnectOptions::from_str(url).map_err(|error| IoError(error.to_string()))?;
        let pool = MySqlPool::connect_with(options)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let connection = Connection {
            url: url.to_string(),
            pool,
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
        let rows = sqlx::query(sql)
            .execute(&self.pool)
            .await
            .map_err(|error| IoError(error.to_string()))?
            .rows_affected();
        Ok(rows)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let query_rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let columns: Vec<String> = query_rows
            .first()
            .map(|row| {
                row.columns()
                    .iter()
                    .map(|column| column.name().to_string())
                    .collect()
            })
            .unwrap_or_default();

        let mut rows = Vec::new();
        for row in query_rows {
            let mut row_data = Vec::new();
            for column in row.columns() {
                let value = Self::convert_to_value(&row, column)?;
                row_data.push(value);
            }
            rows.push(row_data);
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(MySqlDialect {})
    }
}

impl Connection {
    fn convert_to_value(row: &MySqlRow, column: &MySqlColumn) -> Result<Value> {
        let column_name = column.name();

        if let Ok(value) = row.try_get::<Option<String>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::String(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<Vec<u8>>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Bytes(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i16>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I16(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i32>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I32(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<i64>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::I64(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<f32>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::F32(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<f64>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::F64(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<rust_decimal::Decimal>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Decimal(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<bool>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Bool(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<NaiveDate>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Date(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<NaiveTime>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::Time(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<NaiveDateTime>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::DateTime(v)),
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<OffsetDateTime>, &str>(column_name) {
            match value {
                Some(v) => {
                    let date = v.date();
                    let time = v.time();
                    let date_time_string = format!("{date} {time}");
                    let date_time =
                        NaiveDateTime::parse_from_str(&date_time_string, "%Y-%m-%d %H:%M:%S%.f")
                            .expect("invalid date");
                    Ok(Value::DateTime(date_time))
                }
                None => Ok(Value::Null),
            }
        } else if let Ok(value) = row.try_get::<Option<serde_json::Value>, &str>(column_name) {
            match value {
                Some(v) => Ok(Value::from(v)),
                None => Ok(Value::Null),
            }
        } else {
            let column_type = column.type_info();
            let type_name = format!("{column_type:?}");
            return Err(UnsupportedColumnType {
                column_name: column_name.to_string(),
                column_type: type_name,
            });
        }
    }
}
