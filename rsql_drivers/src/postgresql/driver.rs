use crate::error::Result;
use crate::postgresql::metadata;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, Metadata, QueryResult};
use async_trait::async_trait;
use bit_vec::BitVec;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
use postgresql_archive::Version;
use postgresql_embedded::{PostgreSQL, Settings, Status};
use sqlx::postgres::types::Oid;
use sqlx::postgres::{PgColumn, PgConnectOptions, PgRow};
use sqlx::{Column, ColumnIndex, Decode, PgPool, Row, Type};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;
use tracing::debug;
use url::Url;

const POSTGRESQL_EMBEDDED_VERSION: &str = "16.3.0";

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "postgresql"
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
    postgresql: Option<PostgreSQL>,
    pool: PgPool,
}

impl Connection {
    pub(crate) async fn new(url: String, password: Option<String>) -> Result<Connection> {
        let parsed_url = Url::parse(url.as_str())?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();
        let embedded = query_parameters
            .get("embedded")
            .map(|v| v == "true")
            .unwrap_or(false);
        let mut database_url = url.to_string();

        let postgresql = if embedded {
            let default_version = POSTGRESQL_EMBEDDED_VERSION.to_string();
            let specified_version = query_parameters.get("version").unwrap_or(&default_version);
            let version = Version::from_str(specified_version)?;
            let mut settings = Settings::from_url(url)?;

            if let Some(config_dir) = query_parameters.get("installation_dir") {
                settings.installation_dir = PathBuf::from(config_dir);
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

        let options = PgConnectOptions::from_str(database_url.as_str())?;
        let pool = PgPool::connect_with(options).await?;
        let connection = Connection { postgresql, pool };

        Ok(connection)
    }
}

#[async_trait]
impl crate::Connection for Connection {
    async fn execute(&mut self, sql: &str) -> Result<u64> {
        let rows = sqlx::query(sql).execute(&self.pool).await?.rows_affected();
        Ok(rows)
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        let query_rows = sqlx::query(sql).fetch_all(&self.pool).await?;
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
                let value = self.convert_to_value(&row, column)?;
                row_data.push(value);
            }
            rows.push(crate::Row::new(row_data));
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;

        if let Some(postgresql) = &self.postgresql {
            if postgresql.status() == Status::Started {
                if let Err(error) = postgresql.stop().await {
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }
}

impl Connection {
    fn convert_to_value(&self, row: &PgRow, column: &PgColumn) -> Result<Value> {
        let column_type = column.type_info();
        let postgresql_type = column_type.deref();
        let column_type = format!("{:?}", postgresql_type);
        let column_type_parts: Vec<&str> = column_type.split('(').collect();
        let column_name = column.name();

        let value = match column_type_parts[0] {
            "Bool" => self.get_value(row, column_name, |v: bool| Value::Bool(v))?,
            "BoolArray" => self.get_value(row, column_name, |v: Vec<bool>| {
                Value::Array(v.into_iter().map(Value::Bool).collect())
            })?,
            "Bpchar" | "Char" | "Name" | "Text" | "Varchar" => {
                self.get_value(row, column_name, |v: String| Value::String(v))?
            }
            "BpcharArray" | "CharArray" | "NameArray" | "TextArray" | "VarcharArray" => self
                .get_value(row, column_name, |v: Vec<String>| {
                    Value::Array(v.into_iter().map(Value::String).collect())
                })?,
            "Bytea" => self.get_value(row, column_name, |v: Vec<u8>| Value::Bytes(v.to_vec()))?,
            "ByteaArray" => self.get_value(row, column_name, |v: Vec<Vec<u8>>| {
                Value::Array(v.into_iter().map(|v| Value::Bytes(v.to_vec())).collect())
            })?,
            "Int2" => self.get_value(row, column_name, |v: i16| Value::I16(v))?,
            "Int2Array" => self.get_value(row, column_name, |v: Vec<i16>| {
                Value::Array(v.into_iter().map(Value::I16).collect())
            })?,
            "Int4" => self.get_value(row, column_name, |v: i32| Value::I32(v))?,
            "Int4Array" => self.get_value(row, column_name, |v: Vec<i32>| {
                Value::Array(v.into_iter().map(Value::I32).collect())
            })?,
            "Int8" => self.get_value(row, column_name, |v: i64| Value::I64(v))?,
            "Int8Array" => self.get_value(row, column_name, |v: Vec<i64>| {
                Value::Array(v.into_iter().map(Value::I64).collect())
            })?,
            "Oid" => self.get_value(row, column_name, |v: Oid| Value::U32(v.0))?,
            "OidArray" => self.get_value(row, column_name, |v: Vec<Oid>| {
                Value::Array(v.into_iter().map(|v| Value::U32(v.0)).collect())
            })?,
            "Json" | "Jsonb" => {
                self.get_value(row, column_name, |v: serde_json::Value| Value::Json(v))?
            }
            "JsonArray" | "JsonbArray" => {
                self.get_value(row, column_name, |v: Vec<serde_json::Value>| {
                    Value::Array(v.into_iter().map(Value::Json).collect())
                })?
            }
            // "Point" => Value::Null,
            // "PointArray" => Value::Null,
            // "Lseg" => Value::Null,
            // "LsegArray" => Value::Null,
            // "Path" => Value::Null,
            // "PathArray" => Value::Null,
            // "Box" => Value::Null,
            // "BoxArray" => Value::Null,
            // "Polygon" => Value::Null,
            // "PolygonArray" => Value::Null,
            // "Line" => Value::Null,
            // "LineArray" => Value::Null,
            // "Cidr" => Value::Null,
            // "CidrArray" => Value::Null,
            "Float4" => self.get_value(row, column_name, |v: f32| Value::F32(v))?,
            "Float4Array" => self.get_value(row, column_name, |v: Vec<f32>| {
                Value::Array(v.into_iter().map(Value::F32).collect())
            })?,
            "Float8" => self.get_value(row, column_name, |v: f64| Value::F64(v))?,
            "Float8Array" => self.get_value(row, column_name, |v: Vec<f64>| {
                Value::Array(v.into_iter().map(Value::F64).collect())
            })?,
            // "Unknown" => Value::Null,
            // "Circle" => Value::Null,
            // "CircleArray" => Value::Null,
            // "Macaddr" => Value::Null,
            // "MacaddrArray" => Value::Null,
            // "Macaddr8" => Value::Null,
            // "Macaddr8Array" => Value::Null,
            // "Inet" => Value::Null,
            // "InetArray" => Value::Null,
            "Date" => self.get_value(row, column_name, |v: NaiveDate| Value::Date(v))?,
            "DateArray" => self.get_value(row, column_name, |v: Vec<NaiveDate>| {
                Value::Array(v.into_iter().map(Value::Date).collect())
            })?,
            "Time" | "Timetz" => self.get_value(row, column_name, |v: NaiveTime| Value::Time(v))?,
            "TimeArray" | "TimetzArray" => {
                self.get_value(row, column_name, |v: Vec<NaiveTime>| {
                    Value::Array(v.into_iter().map(Value::Time).collect())
                })?
            }
            "Timestamp" => {
                self.get_value(row, column_name, |v: NaiveDateTime| Value::DateTime(v))?
            }
            "TimestampArray" => self.get_value(row, column_name, |v: Vec<NaiveDateTime>| {
                Value::Array(v.into_iter().map(Value::DateTime).collect())
            })?,
            "Timestamptz" => self.get_value(row, column_name, |v: chrono::DateTime<Utc>| {
                Value::DateTime(v.naive_utc())
            })?,
            "TimestamptzArray" => {
                self.get_value(row, column_name, |v: Vec<chrono::DateTime<Utc>>| {
                    Value::Array(
                        v.into_iter()
                            .map(|v| Value::DateTime(v.naive_utc()))
                            .collect(),
                    )
                })?
            }
            // "Interval" => Value::Null,
            // "IntervalArray" => Value::Null,
            "Bit" | "Varbit" => self.get_value(row, column_name, |v: BitVec| {
                Value::String(self.bit_string(v))
            })?,
            "BitArray" | "VarbitArray" => self.get_value(row, column_name, |v: Vec<BitVec>| {
                Value::Array(
                    v.into_iter()
                        .map(|v| Value::String(self.bit_string(v)))
                        .collect(),
                )
            })?,
            "Numeric" => self.get_value(row, column_name, |v: rust_decimal::Decimal| {
                Value::String(v.to_string())
            })?,
            "NumericArray" => {
                self.get_value(row, column_name, |v: Vec<rust_decimal::Decimal>| {
                    Value::Array(
                        v.into_iter()
                            .map(|v| Value::String(v.to_string()))
                            .collect(),
                    )
                })?
            }
            // "Record" => Value::Null,
            // "RecordArray" => Value::Null,
            "Uuid" => self.get_value(row, column_name, |v: uuid::Uuid| Value::Uuid(v))?,
            "UuidArray" => self.get_value(row, column_name, |v: Vec<uuid::Uuid>| {
                Value::Array(v.into_iter().map(Value::Uuid).collect())
            })?,
            // "Int4Range" => Value::Null,
            // "Int4RangeArray" => Value::Null,
            // "NumRange" => Value::Null,
            // "NumRangeArray" => Value::Null,
            // "TsRange" => Value::Null,
            // "TsRangeArray" => Value::Null,
            // "TstzRange" => Value::Null,
            // "TstzRangeArray" => Value::Null,
            // "DateRange" => Value::Null,
            // "DateRangeArray" => Value::Null,
            // "Int8Range" => Value::Null,
            // "Int8RangeArray" => Value::Null,
            // "Jsonpath" => Value::Null,
            // "JsonpathArray" => Value::Null,
            // "Money" => Value::Null,
            // "MoneyArray" => Value::Null,
            "Void" => Value::Null, // pg_sleep() returns void
            // "Custom" => Value::Null,
            // "DeclareWithName" => Value::Null,
            // "DeclareWithOid" => Value::Null,
            _ => {
                return Err(UnsupportedColumnType {
                    column_name: column.name().to_string(),
                    column_type: column_type.to_string(),
                });
            }
        };

        Ok(value)
    }

    fn get_value<'r, T, I>(
        &self,
        row: &'r PgRow,
        index: I,
        to_value: impl Fn(T) -> Value,
    ) -> Result<Value>
    where
        T: Decode<'r, <PgRow as Row>::Database> + Type<<PgRow as Row>::Database>,
        I: ColumnIndex<PgRow>,
    {
        match row.try_get::<Option<T>, I>(index)?.map(to_value) {
            Some(value) => Ok(value),
            None => Ok(Value::Null),
        }
    }

    fn bit_string(&self, value: BitVec) -> String {
        let bit_string: String = value
            .iter()
            .map(|bit| if bit { '1' } else { '0' })
            .collect();
        bit_string
    }
}

// postgresql embedded is not functioning on Windows yet
#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod test {
    use crate::{DriverManager, Row, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use serde_json::json;
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    use testcontainers::runners::AsyncRunner;

    const DATABASE_URL: &str = "postgresql://?embedded=true";

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE person (id INTEGER, name VARCHAR(20))")
            .await?;

        let rows = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection.query("SELECT id, name FROM person").await?;
        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(
            query_result.next().await,
            Some(Row::new(vec![
                Value::I32(1),
                Value::String("foo".to_string())
            ]))
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    async fn test_data_type(sql: &str) -> anyhow::Result<Option<Value>> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let mut query_result = connection.query(sql).await?;
        let mut value: Option<Value> = None;

        assert_eq!(query_result.columns().await.len(), 1);

        if let Some(row) = query_result.next().await {
            assert_eq!(row.len(), 1);

            value = row.get(0).cloned();
        }
        assert!(query_result.next().await.is_none());

        connection.close().await?;
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

        let result =
            test_data_type("SELECT ARRAY[CAST(B'10' as bit(2)), CAST(B'101' as bit(3))]").await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::String("10".to_string()));
            assert_eq!(value[1], Value::String("101".to_string()));
        }

        let result = test_data_type("SELECT CAST(B'10101' as bit varying(5))").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("10101".to_string()));

        let result = test_data_type(
            "SELECT ARRAY[CAST(B'10' as bit varying(5)), CAST(B'101' as bit varying(5))]",
        )
        .await?;
        assert!(result.is_some());
        if let Some(Value::Array(value)) = result {
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], Value::String("10".to_string()));
            assert_eq!(value[1], Value::String("101".to_string()));
        }

        let result = test_data_type("SELECT CAST(1.234 as numeric)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("1.234".to_string()));

        let result = test_data_type("SELECT CAST(1.234 as decimal)").await?;
        let value = result.expect("value is None");
        assert_eq!(value, Value::String("1.234".to_string()));
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
        let date_time = NaiveDateTime::parse_from_str("1983-01-01 01:23:45", "%Y-%m-%d %H:%M:%S")?;
        assert_eq!(value, Value::DateTime(date_time));

        let now = Utc::now().naive_utc();
        let result = test_data_type("SELECT now()").await?;
        let value = result.expect("value is None");
        if let Value::DateTime(value) = value {
            assert!(value > now);
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
    async fn test_data_type_null() -> anyhow::Result<()> {
        let result = test_data_type("SELECT pg_sleep(0)").await?;
        assert_eq!(result, Some(Value::Null));
        Ok(())
    }

    #[tokio::test]
    async fn test_data_type_not_supported() -> anyhow::Result<()> {
        let result = test_data_type("SELECT CAST('<a>b</a> as xml)").await;
        assert!(result.is_err());
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let postgres_image = testcontainers::ContainerRequest::from(
            testcontainers_modules::postgres::Postgres::default(),
        );
        let container = postgres_image.start().await?;
        let port = container.get_host_port_ipv4(5432).await?;

        let database_url = format!("postgresql://postgres:postgres@localhost:{}/postgres", port);
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        let mut query_result = connection.query("SELECT 'foo'::TEXT").await?;
        let row = query_result.next().await.expect("no row");
        let value = row.first().expect("no value");

        assert_eq!(*value, Value::String("foo".to_string()));

        Ok(())
    }
}
