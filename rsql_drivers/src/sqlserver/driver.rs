use crate::error::Result;
use crate::sqlserver::metadata;
use crate::value::Value;
use crate::Error::UnsupportedColumnType;
use crate::{MemoryQueryResult, Metadata, QueryResult};
use async_trait::async_trait;
use futures_util::stream::TryStreamExt;
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
        let connection = Connection { client };

        Ok(connection)
    }
}

#[async_trait]
impl crate::Connection for Connection {
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
                rows.push(crate::Row::new(row_data));
            }
        }

        let query_result = MemoryQueryResult::new(columns, rows);
        Ok(Box::new(query_result))
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

fn convert_to_value(row: &Row, column: &Column, index: usize) -> Result<Option<Value>> {
    let column_name = column.name();

    if let Ok(value) = row.try_get(index) {
        let value: Option<&str> = value;
        Ok(value.map(|v| Value::String(v.to_string())))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<&[u8]> = value;
        Ok(value.map(|v| Value::Bytes(v.to_vec())))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<u8> = value;
        Ok(value.map(Value::U8))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i16> = value;
        Ok(value.map(Value::I16))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i32> = value;
        Ok(value.map(Value::I32))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i64> = value;
        Ok(value.map(Value::I64))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<f32> = value;
        Ok(value.map(Value::F32))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<f64> = value;
        Ok(value.map(Value::F64))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<bool> = value;
        Ok(value.map(Value::Bool))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<rust_decimal::Decimal> = value;
        Ok(value.map(|v| Value::String(v.to_string())))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDate> = value;
        Ok(value.map(Value::Date))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveTime> = value;
        Ok(value.map(Value::Time))
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDateTime> = value;
        Ok(value.map(Value::DateTime))
    } else {
        let column_type = format!("{:?}", column.column_type());
        let type_name = format!("{:?}", column_type);
        return Err(UnsupportedColumnType {
            column_name: column_name.to_string(),
            column_type: type_name,
        });
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod test {
    use super::*;
    use crate::{Connection, DriverManager, Value};
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use indoc::indoc;
    use testcontainers::clients::Cli;
    use testcontainers::RunnableImage;
    use testcontainers_modules::mssql_server::MssqlServer;

    const PASSWORD: &str = "Password42!";

    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let docker = Cli::default();
        let sqlserver_image =
            RunnableImage::from(MssqlServer::default().with_sa_password(PASSWORD));
        let container = docker.run(sqlserver_image);
        let port = container.get_host_port_ipv4(1433);
        let database_url =
            &format!("sqlserver://sa:{PASSWORD}@127.0.0.1:{port}?TrustServerCertificate=true");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        test_connection_interface(&mut *connection).await?;
        test_data_types(&mut *connection).await?;

        Ok(())
    }

    async fn test_connection_interface(connection: &mut dyn Connection) -> anyhow::Result<()> {
        let _ = connection
            .execute("CREATE TABLE person (id INT, name VARCHAR(20))")
            .await?;

        let rows = connection
            .execute("INSERT INTO person (id, name) VALUES (1, 'foo')")
            .await?;
        assert_eq!(rows, 1);

        let mut query_result = connection.query("SELECT id, name FROM person").await?;
        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        match query_result.next().await {
            Some(row) => {
                assert_eq!(row.len(), 2);

                if let Some(Value::I32(id)) = row.get(0) {
                    assert_eq!(*id, 1);
                } else {
                    assert!(false);
                }

                if let Some(Value::String(name)) = row.get(1) {
                    assert_eq!(name, "foo");
                } else {
                    assert!(false);
                }
            }
            None => assert!(false),
        }
        assert!(query_result.next().await.is_none());
        Ok(())
    }

    async fn test_data_types(connection: &mut dyn Connection) -> anyhow::Result<()> {
        let sql = indoc! {r#"
            CREATE TABLE data_types (
                nchar_type NCHAR(1),
                char_type CHAR(1),
                nvarchar_type NVARCHAR(50),
                varchar_type VARCHAR(50),
                ntext_type NTEXT,
                text_type TEXT,
                binary_type BINARY(1),
                varbinary_type VARBINARY(1),
                tinyint_type TINYINT,
                smallint_type SMALLINT,
                int_type INT,
                bigint_type BIGINT,
                float24_type FLOAT(24),
                float53_type FLOAT(53),
                bit_type BIT,
                decimal_type DECIMAL(5,2),
                date_type DATE,
                time_type TIME,
                datetime_type DATETIME
            )
        "#};
        let _ = connection.execute(sql).await?;

        let sql = indoc! {r#"
            INSERT INTO data_types (
                nchar_type, char_type, nvarchar_type, varchar_type, ntext_type, text_type,
                binary_type, varbinary_type,
                tinyint_type, smallint_type, int_type, bigint_type,
                float24_type, float53_type, bit_type, decimal_type,
                date_type, time_type, datetime_type
            ) VALUES (
                 'a', 'a', 'foo', 'foo', 'foo', 'foo',
                 CAST(42 AS BINARY(1)), CAST(42 AS VARBINARY(1)),
                 127, 32767, 2147483647, 9223372036854775807,
                 123.45, 123.0, 1, 123.0,
                 '2022-01-01', '14:30:00', '2022-01-01 14:30:00'
             )
        "#};
        let _ = connection.execute(sql).await?;

        let sql = indoc! {r#"
            SELECT nchar_type, char_type, nvarchar_type, varchar_type, ntext_type, text_type,
                   binary_type, varbinary_type,
                   tinyint_type, smallint_type, int_type, bigint_type,
                   float24_type, float53_type, bit_type, decimal_type,
                   date_type, time_type, datetime_type
              FROM data_types
        "#};
        let mut query_result = connection.query(sql).await?;

        if let Some(row) = query_result.next().await {
            assert_eq!(row.get(0).cloned().unwrap(), Value::String("a".to_string()));
            assert_eq!(row.get(1).cloned().unwrap(), Value::String("a".to_string()));
            assert_eq!(
                row.get(2).cloned().unwrap(),
                Value::String("foo".to_string())
            );
            assert_eq!(
                row.get(3).cloned().unwrap(),
                Value::String("foo".to_string())
            );
            assert_eq!(
                row.get(4).cloned().unwrap(),
                Value::String("foo".to_string())
            );
            assert_eq!(
                row.get(5).cloned().unwrap(),
                Value::String("foo".to_string())
            );
            assert_eq!(row.get(6).cloned().unwrap(), Value::Bytes(vec![42u8]));
            assert_eq!(row.get(7).cloned().unwrap(), Value::Bytes(vec![42u8]));
            assert_eq!(row.get(8).cloned().unwrap(), Value::U8(127));
            assert_eq!(row.get(9).cloned().unwrap(), Value::I16(32_767));
            assert_eq!(row.get(10).cloned().unwrap(), Value::I32(2_147_483_647));
            assert_eq!(
                row.get(11).cloned().unwrap(),
                Value::I64(9_223_372_036_854_775_807)
            );
            assert_eq!(row.get(12).cloned().unwrap(), Value::F32(123.45));
            assert_eq!(row.get(13).cloned().unwrap(), Value::F64(123.0));
            assert_eq!(row.get(14).cloned().unwrap(), Value::Bool(true));
            assert_eq!(
                row.get(15).cloned().unwrap(),
                Value::String("123.00".to_string())
            );
            let date = NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid date");
            assert_eq!(row.get(16).cloned().unwrap(), Value::Date(date));
            let time = NaiveTime::from_hms_opt(14, 30, 00).expect("invalid time");
            assert_eq!(row.get(17).cloned().unwrap(), Value::Time(time));
            let date_time =
                NaiveDateTime::parse_from_str("2022-01-01 14:30:00", "%Y-%m-%d %H:%M:%S")?;
            assert_eq!(row.get(18).cloned().unwrap(), Value::DateTime(date_time));
        }

        Ok(())
    }
}
