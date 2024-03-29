use crate::configuration::Configuration;
use crate::drivers::error::Result;
use crate::drivers::value::Value;
use crate::drivers::Error::UnsupportedColumnType;
use crate::drivers::{MemoryQueryResult, Results};
use async_trait::async_trait;
use bit_vec::BitVec;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use indoc::indoc;
use postgresql_archive::Version;
use postgresql_embedded::{PostgreSQL, Settings};
use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;
use std::time::SystemTime;
use tokio_postgres::types::{FromSql, ToSql, Type};
use tokio_postgres::{Client, Column, NoTls, Row};
use tracing::debug;
use url::Url;

const POSTGRESQL_EMBEDDED_VERSION: &str = "16.2.3";

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::drivers::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "postgres"
    }

    async fn connect(
        &self,
        configuration: &Configuration,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn crate::drivers::Connection>> {
        let connection = Connection::new(configuration, url, password).await?;
        Ok(Box::new(connection))
    }
}

#[derive(Debug)]
pub(crate) struct Connection {
    postgresql: Option<PostgreSQL>,
    client: Client,
}

impl Connection {
    pub(crate) async fn new(
        configuration: &Configuration,
        url: String,
        password: Option<String>,
    ) -> Result<Connection> {
        let parsed_url = Url::parse(url.as_str())?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();
        let embedded = query_parameters
            .get("embedded")
            .map(|v| v == "true")
            .unwrap_or(false);
        let mut database_url = url.to_string().replace("postgres://", "postgresql://");

        let postgresql = if embedded {
            let default_version = POSTGRESQL_EMBEDDED_VERSION.to_string();
            let specified_version = query_parameters.get("version").unwrap_or(&default_version);
            let version = Version::from_str(specified_version)?;
            let mut settings = Settings::from_url(url)?;

            if let Some(config_dir) = &configuration.config_dir {
                settings.installation_dir = config_dir.clone();
            }
            if let Some(password) = password {
                settings.password = password;
            }

            debug!("Starting embedded PostgreSQL {version} server");
            let mut postgresql = PostgreSQL::new(version, settings);
            postgresql.setup().await?;
            postgresql.start().await?;

            let database_name = "embedded";
            postgresql.create_database(database_name).await?;
            let settings = postgresql.settings();
            database_url = settings.url(database_name);
            Some(postgresql)
        } else {
            None
        };

        let (client, connection) = tokio_postgres::connect(database_url.as_str(), NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let connection = Connection { postgresql, client };

        Ok(connection)
    }
}

#[async_trait]
impl crate::drivers::Connection for Connection {
    async fn execute(&self, sql: &str) -> Result<Results> {
        let rows = self.client.execute(sql, &[]).await?;
        Ok(Results::Execute(rows))
    }

    async fn indexes<'table>(&mut self, table: Option<&'table str>) -> Result<Vec<String>> {
        let mut sql = indoc! {r#"
            SELECT i.relname AS index_name
              FROM pg_class t,
                   pg_class i,
                   pg_index ix,
                   pg_attribute a,
                   information_schema.tables ist
             WHERE t.oid = ix.indrelid
               AND i.oid = ix.indexrelid
               AND a.attrelid = t.oid
               AND a.attnum = ANY(ix.indkey)
               AND t.relkind = 'r'
               AND ist.table_name = t.relname
               AND ist.table_schema = current_schema()
        "#}
        .to_string();
        if table.is_some() {
            sql = format!("{sql} AND ist.table_name = $1");
        }
        sql = format!("{sql} ORDER BY index_name").to_string();
        let query_rows = match table {
            Some(table) => {
                let table: &(dyn ToSql + Sync) = &table;
                self.client.query(sql.as_str(), &[table]).await?
            }
            None => self.client.query(sql.as_str(), &[]).await?,
        };
        let mut indexes = Vec::new();

        for row in query_rows {
            if let Some(column) = row.columns().first() {
                if let Some(value) = self.convert_to_value(&row, column, 0)? {
                    indexes.push(value.to_string());
                }
            }
        }

        Ok(indexes)
    }

    async fn query(&self, sql: &str, limit: u64) -> Result<Results> {
        let statement = self.client.prepare(sql).await?;
        let query_columns = statement.columns();
        let columns: Vec<String> = query_columns
            .iter()
            .map(|column| column.name().to_string())
            .collect();

        let query_rows = self.client.query(sql, &[]).await?;
        let mut rows = Vec::new();
        for query_row in query_rows {
            let mut row = Vec::new();
            for (index, column) in query_columns.iter().enumerate() {
                let value = self.convert_to_value(&query_row, column, index)?;
                row.push(value);
            }
            rows.push(row);

            if limit > 0 && rows.len() >= limit as usize {
                break;
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Results::Query(Box::new(query_result)))
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = indoc! { r#"
            SELECT table_name
              FROM information_schema.tables
             WHERE table_catalog = current_database()
               AND table_schema = 'public'
             ORDER BY table_name
        "#};
        let results = self.query(sql, 0).await?;
        let mut tables = Vec::new();

        if let Results::Query(query_results) = results {
            for row in query_results.rows().await {
                if let Some(data) = &row[0] {
                    tables.push(data.to_string());
                }
            }
        }

        Ok(tables)
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(postgresql) = &self.postgresql {
            match postgresql.stop().await {
                Ok(_) => Ok(()),
                Err(error) => Err(error.into()),
            }
        } else {
            Ok(())
        }
    }
}

impl Connection {
    pub(crate) fn convert_to_value(
        &self,
        row: &Row,
        column: &Column,
        column_index: usize,
    ) -> Result<Option<Value>> {
        // https://www.postgresql.org/docs/current/datatype.html
        let column_type = column.type_();
        let value = match *column_type {
            Type::BIT | Type::VARBIT => {
                let bits_value: Option<BitVec> = row.try_get(column_index)?;
                match bits_value {
                    Some(value) => {
                        let bit_string: String = value
                            .iter()
                            .map(|bit| if bit { '1' } else { '0' })
                            .collect();
                        Some(Value::String(bit_string))
                    }
                    None => None,
                }
            }
            Type::BOOL => self.get_single(row, column_index, |v: bool| Value::Bool(v))?,
            Type::BOOL_ARRAY => self.get_array(row, column_index, |v: bool| Value::Bool(v))?,
            Type::INT2 => self.get_single(row, column_index, |v: i16| Value::I16(v))?,
            Type::INT2_ARRAY => self.get_array(row, column_index, |v: i16| Value::I16(v))?,
            Type::INT4 => self.get_single(row, column_index, |v: i32| Value::I32(v))?,
            Type::INT4_ARRAY => self.get_array(row, column_index, |v: i32| Value::I32(v))?,
            Type::INT8 => self.get_single(row, column_index, |v: i64| Value::I64(v))?,
            Type::INT8_ARRAY => self.get_array(row, column_index, |v: i64| Value::I64(v))?,
            Type::FLOAT4 => self.get_single(row, column_index, |v: f32| Value::F32(v))?,
            Type::FLOAT4_ARRAY => self.get_array(row, column_index, |v: f32| Value::F32(v))?,
            Type::FLOAT8 => self.get_single(row, column_index, |v: f64| Value::F64(v))?,
            Type::FLOAT8_ARRAY => self.get_array(row, column_index, |v: f64| Value::F64(v))?,
            Type::TEXT | Type::VARCHAR | Type::CHAR | Type::BPCHAR | Type::NAME => {
                self.get_single(row, column_index, |v: String| Value::String(v))?
            }
            Type::TEXT_ARRAY | Type::VARCHAR_ARRAY | Type::CHAR_ARRAY | Type::BPCHAR_ARRAY => {
                self.get_array(row, column_index, |v: String| Value::String(v))?
            }
            Type::JSON | Type::JSONB => {
                self.get_single(row, column_index, |v: serde_json::Value| Value::Json(v))?
            }
            Type::JSON_ARRAY | Type::JSONB_ARRAY => {
                self.get_array(row, column_index, |v: serde_json::Value| Value::Json(v))?
            }
            Type::BYTEA => {
                let byte_value: Option<&[u8]> = row.try_get(column_index)?;
                byte_value.map(|value| Value::Bytes(value.to_vec()))
            }
            Type::DATE => self.get_single(row, column_index, |v: NaiveDate| Value::Date(v))?,
            Type::TIME | Type::TIMETZ => {
                self.get_single(row, column_index, |v: NaiveTime| Value::Time(v))?
            }
            Type::TIMESTAMP => {
                self.get_single(row, column_index, |v: NaiveDateTime| Value::DateTime(v))?
            }
            Type::TIMESTAMPTZ => {
                let system_time: Option<SystemTime> = row.try_get(column_index)?;
                match system_time {
                    Some(value) => {
                        let date_time: DateTime<Utc> = value.into();
                        Some(Value::DateTime(date_time.naive_utc()))
                    }
                    None => None,
                }
            }
            Type::OID => self.get_single(row, column_index, |v: u32| Value::U32(v))?,
            Type::OID_ARRAY => self.get_array(row, column_index, |v: u32| Value::U32(v))?,
            Type::VOID => None, // pg_sleep() returns void
            _ => {
                return Err(UnsupportedColumnType {
                    column_name: column.name().to_string(),
                    column_type: column_type.name().to_string(),
                });
            }
        };

        Ok(value)
    }

    fn get_single<'a, T: FromSql<'a>>(
        &self,
        row: &'a Row,
        column_index: usize,
        to_value: impl Fn(T) -> Value,
    ) -> Result<Option<Value>> {
        let value = row.try_get::<_, Option<T>>(column_index)?.map(to_value);
        Ok(value)
    }

    fn get_array<'a, T: FromSql<'a>>(
        &self,
        row: &'a Row,
        column_index: usize,
        to_value: impl Fn(T) -> Value,
    ) -> Result<Option<Value>> {
        let original_value_array = row.try_get::<_, Option<Vec<T>>>(column_index)?;
        let result = match original_value_array {
            Some(value_array) => {
                let mut values = vec![];
                for value in value_array {
                    values.push(to_value(value));
                }
                Some(Value::Array(values))
            }
            None => None,
        };
        Ok(result)
    }
}

// postgresql embedded is not functioning on Windows yet
#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod test {
    use crate::configuration::Configuration;
    use crate::drivers::{DriverManager, Results, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use serde_json::json;

    const DATABASE_URL: &str = "postgres://?embedded=true";

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;
        connection.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_limit_rows() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let connection = driver_manager.connect(&configuration, DATABASE_URL).await?;
        let results = connection.query("SELECT 1 UNION ALL SELECT 2", 1).await?;
        assert!(results.is_query());
        if let Results::Query(query_result) = results {
            assert_eq!(query_result.rows().await.len(), 1);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE person (id INTEGER, name VARCHAR(20))")
            .await?;

        let execute_results = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        if let Results::Execute(rows) = execute_results {
            assert_eq!(rows, 1);
        }

        let results = connection.query("SELECT id, name FROM person", 0).await?;
        if let Results::Query(query_result) = results {
            assert_eq!(query_result.columns().await, vec!["id", "name"]);
            assert_eq!(query_result.rows().await.len(), 1);
            match query_result.rows().await.get(0) {
                Some(row) => {
                    assert_eq!(row.len(), 2);

                    if let Some(Value::I32(id)) = &row[0] {
                        assert_eq!(*id, 1);
                    } else {
                        assert!(false);
                    }

                    if let Some(Value::String(name)) = &row[1] {
                        assert_eq!(name, "foo");
                    } else {
                        assert!(false);
                    }
                }
                None => assert!(false),
            }
        }

        connection.stop().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> anyhow::Result<Option<Value>> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;

        let results = connection.query(sql, 0).await?;
        let mut value: Option<Value> = None;

        if let Results::Query(query_result) = results {
            assert_eq!(query_result.columns().await.len(), 1);
            assert_eq!(query_result.rows().await.len(), 1);

            if let Some(row) = query_result.rows().await.get(0) {
                assert_eq!(row.len(), 1);

                value = row[0].clone();
            }
        }

        connection.stop().await?;
        Ok(value)
    }

    #[tokio::test]
    async fn test_data_type_bytes() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('1' as bytea)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::Bytes(vec![49]));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_string() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('foo' as char(3))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT CAST('foo' as varchar(5))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT 'foo'::TEXT").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("foo".to_string()));

        let result = test_data_type("SELECT ARRAY['foo','bar']::TEXT[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::String("foo".to_string()));
            assert_eq!(value[1], Value::String("bar".to_string()));
        }

        let result = test_data_type("SELECT CAST(B'101' as bit(3))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("101".to_string()));

        let result = test_data_type("SELECT CAST(B'10101' as bit varying(5))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("10101".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i16() -> anyhow::Result<()> {
        let result = test_data_type("SELECT 32767::INT2").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I16(32_767));

        let result = test_data_type("SELECT ARRAY[0,32767]::INT2[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::I16(0));
            assert_eq!(value[1], Value::I16(32_767));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i32() -> anyhow::Result<()> {
        let result = test_data_type("SELECT 2147483647::INT4").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I32(2_147_483_647));

        let result = test_data_type("SELECT ARRAY[0,2147483647]::INT4[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::I32(0));
            assert_eq!(value[1], Value::I32(2_147_483_647));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_i64() -> anyhow::Result<()> {
        let result = test_data_type("SELECT 2147483647::INT8").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::I64(2_147_483_647));

        let result = test_data_type("SELECT ARRAY[0,2147483647]::INT8[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::I64(0));
            assert_eq!(value[1], Value::I64(2_147_483_647));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_bool() -> anyhow::Result<()> {
        let result = test_data_type("SELECT 1::BOOL").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::Bool(true));

        let result = test_data_type("SELECT ARRAY[0,1]::BOOL[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::Bool(false));
            assert_eq!(value[1], Value::Bool(true));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f32() -> anyhow::Result<()> {
        let result = test_data_type("SELECT 1.234::FLOAT4").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::F32(1.234));

        let result = test_data_type("SELECT ARRAY[0,1.234]::FLOAT4[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::F32(0.0));
            assert_eq!(value[1], Value::F32(1.234));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_f64() -> anyhow::Result<()> {
        let result = test_data_type("SELECT 1.234::FLOAT8").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::F64(1.234));

        let result = test_data_type("SELECT ARRAY[0,1.234]::FLOAT8[]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::F64(0.0));
            assert_eq!(value[1], Value::F64(1.234));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_date() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('1983-01-01' as date)").await?;
        let value = result.expect("value is None");
        let date = NaiveDate::from_ymd_opt(1983, 1, 1).expect("invalid date");
        assert_eq!(value, Value::Date(date));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_time() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('1:23:45' as time)").await?;
        let value = result.expect("value is None");
        let time = NaiveTime::from_hms_opt(1, 23, 45).expect("invalid time");
        assert_eq!(value, Value::Time(time));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_date_time() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('1983-01-01 1:23:45' as timestamp)").await?;
        let value = result.expect("value is None");
        let time = NaiveDateTime::parse_from_str("1983-01-01 01:23:45", "%Y-%m-%d %H:%M:%S")?;
        assert_eq!(value, Value::DateTime(time));

        let now = Utc::now().naive_utc();
        let result = test_data_type("SELECT now()").await?;
        let value = result.expect("value is None");
        if let Value::DateTime(value) = value {
            assert!(value > now);
        } else {
            assert!(false);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_json() -> anyhow::Result<()> {
        let result = test_data_type(r#"SELECT CAST('{"key": "value"}' as json)"#).await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::Json(json!({"key": "value"})));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_none() -> anyhow::Result<()> {
        let result = test_data_type("SELECT pg_sleep(0)").await?;
        assert_eq!(result, None);
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_not_supported() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('<a>b</a> as xml)").await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_indexes() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;

        let tables = connection.indexes(None).await?;
        assert_eq!(tables, vec!["contacts_pkey", "users_pkey"]);

        connection.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_indexes_table() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;

        let tables = connection.indexes(Some("users")).await?;
        assert_eq!(tables, vec!["users_pkey"]);

        connection.stop().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_tables() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&configuration, DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;

        let tables = connection.tables().await?;
        assert_eq!(tables, vec!["contacts", "users"]);

        connection.stop().await?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let docker = testcontainers::clients::Cli::default();
        let postgres_image = testcontainers::RunnableImage::from(
            testcontainers_modules::postgres::Postgres::default(),
        );
        let container = docker.run(postgres_image);
        let port = container.get_host_port_ipv4(5432);

        let database_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
        let configuration = Configuration::default();
        let driver_manager = DriverManager::default();
        let connection = driver_manager
            .connect(&configuration, database_url.as_str())
            .await?;

        let results = connection.query("SELECT 'foo'::TEXT", 0).await?;
        assert!(results.is_query());
        if let Results::Query(query_result) = results {
            let rows = query_result.rows().await;
            let row = rows.first().expect("row is None");
            let cell = row.first().expect("cell is None");
            if let Some(value) = cell.clone() {
                assert_eq!(value, Value::String("foo".to_string()));
            }
        }

        Ok(())
    }
}
