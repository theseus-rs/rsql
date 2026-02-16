use crate::metadata;
use crate::results::MySqlQueryResult;
use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{Metadata, QueryResult, Result, ToSql, Value};
use sqlparser::dialect::{Dialect, MySqlDialect};
use sqlx::mysql::{MySqlArguments, MySqlConnectOptions};
use sqlx::{Column, MySql, MySqlPool, Row};
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

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let values = rsql_driver::to_values(params);
        let mut query = sqlx::query(sql);
        for value in &values {
            query = bind_mysql_value(query, value);
        }
        let rows = query
            .execute(&self.pool)
            .await
            .map_err(|error| IoError(error.to_string()))?
            .rows_affected();
        Ok(rows)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let values = rsql_driver::to_values(params);
        let mut query = sqlx::query(sql);
        for value in &values {
            query = bind_mysql_value(query, value);
        }
        let query_rows = query
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

        let query_result = MySqlQueryResult::new(columns, query_rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(MySqlDialect {})
    }
}

fn bind_mysql_value<'q>(
    query: sqlx::query::Query<'q, MySql, MySqlArguments>,
    value: &'q Value,
) -> sqlx::query::Query<'q, MySql, MySqlArguments> {
    match value {
        Value::Null => query.bind(None::<String>),
        Value::Bool(v) => query.bind(*v),
        Value::I8(v) => query.bind(i16::from(*v)),
        Value::I16(v) => query.bind(*v),
        Value::I32(v) => query.bind(*v),
        Value::I64(v) => query.bind(*v),
        Value::U8(v) => query.bind(i16::from(*v)),
        Value::U16(v) => query.bind(i32::from(*v)),
        Value::U32(v) => query.bind(i64::from(*v)),
        Value::U64(v) => query.bind(*v),
        Value::F32(v) => query.bind(*v),
        Value::F64(v) => query.bind(*v),
        Value::String(v) => query.bind(v.as_str()),
        Value::Bytes(v) => query.bind(v.as_slice()),
        _ => query.bind(value.to_string()),
    }
}
