#[cfg(target_os = "linux")]
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
#[cfg(target_os = "linux")]
use indoc::indoc;
#[cfg(target_os = "linux")]
use rsql_driver::{Connection, Driver, Value};
#[cfg(target_os = "linux")]
use std::str::FromStr;
#[cfg(target_os = "linux")]
use testcontainers::ContainerRequest;
#[cfg(target_os = "linux")]
use testcontainers::runners::AsyncRunner;
#[cfg(target_os = "linux")]
use testcontainers_modules::mssql_server::MssqlServer;

#[cfg(target_os = "linux")]
const PASSWORD: &str = "Password42!";

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_sqlserver_driver() -> anyhow::Result<()> {
    let image = ContainerRequest::from(
        MssqlServer::default()
            .with_accept_eula()
            .with_sa_password(PASSWORD),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(1433).await?;
    let database_url =
        format!("sqlserver://sa:{PASSWORD}@127.0.0.1:{port}?TrustServerCertificate=true");
    let mut connection = rsql_driver_sqlserver::Driver
        .connect(database_url.as_str())
        .await?;
    assert_eq!(database_url, connection.url().as_str());

    test_connection_interface(&mut *connection).await?;
    test_data_types(&mut *connection).await?;

    container.stop().await?;
    container.rm().await?;
    Ok(())
}

#[cfg(target_os = "linux")]
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
    assert_eq!(
        query_result.next().await,
        Some(vec![Value::I32(1), Value::String("foo".to_string())])
    );
    assert!(query_result.next().await.is_none());

    let db_metadata = connection.metadata().await?;
    let schema = db_metadata
        .current_schema()
        .expect("expected at least one schema");
    assert!(schema.tables().iter().any(|table| table.name() == "person"));

    Ok(())
}

#[cfg(target_os = "linux")]
async fn test_data_types(connection: &mut dyn Connection) -> anyhow::Result<()> {
    let sql = indoc! {r"
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
    "};
    let _ = connection.execute(sql).await?;

    let sql = indoc! {r"
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
    "};
    let _ = connection.execute(sql).await?;

    let sql = indoc! {r"
        SELECT nchar_type, char_type, nvarchar_type, varchar_type, ntext_type, text_type,
               binary_type, varbinary_type,
               tinyint_type, smallint_type, int_type, bigint_type,
               float24_type, float53_type, bit_type, decimal_type,
               date_type, time_type, datetime_type
          FROM data_types
    "};
    let mut query_result = connection.query(sql).await?;
    assert_eq!(
        query_result.next().await,
        Some(vec![
            Value::String("a".to_string()),
            Value::String("a".to_string()),
            Value::String("foo".to_string()),
            Value::String("foo".to_string()),
            Value::String("foo".to_string()),
            Value::String("foo".to_string()),
            Value::Bytes(vec![42u8]),
            Value::Bytes(vec![42u8]),
            Value::U8(127),
            Value::I16(32_767),
            Value::I32(2_147_483_647),
            Value::I64(9_223_372_036_854_775_807),
            Value::F32(123.45),
            Value::F64(123.0),
            Value::Bool(true),
            Value::Decimal(rust_decimal::Decimal::from_str("123.00").expect("invalid decimal")),
            Value::Date(NaiveDate::from_ymd_opt(2022, 1, 1).expect("invalid date")),
            Value::Time(NaiveTime::from_hms_opt(14, 30, 00).expect("invalid time")),
            Value::DateTime(NaiveDateTime::parse_from_str(
                "2022-01-01 14:30:00",
                "%Y-%m-%d %H:%M:%S"
            )?)
        ])
    );
    assert!(query_result.next().await.is_none());

    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_sqlserver_metadata() -> anyhow::Result<()> {
    let image = ContainerRequest::from(
        MssqlServer::default()
            .with_accept_eula()
            .with_sa_password(PASSWORD),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(1433).await?;
    let database_url =
        format!("sqlserver://sa:{PASSWORD}@127.0.0.1:{port}?TrustServerCertificate=true");
    let mut connection = rsql_driver_sqlserver::Driver
        .connect(database_url.as_str())
        .await?;

    test_schema(&mut *connection).await?;

    container.stop().await?;
    container.rm().await?;
    Ok(())
}

#[cfg(target_os = "linux")]
async fn test_schema(connection: &mut dyn Connection) -> anyhow::Result<()> {
    let _ = connection
        .execute("CREATE TABLE contacts (id INT PRIMARY KEY, email VARCHAR(20))")
        .await?;
    let _ = connection
        .execute("CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(20))")
        .await?;

    let metadata = connection.metadata().await?;
    let schema = metadata.current_schema().expect("schema");

    let contacts_table = schema.get("contacts").expect("contacts table");
    assert_eq!(contacts_table.name(), "contacts");
    assert_eq!(contacts_table.columns().len(), 2);
    let id_column = contacts_table.get_column("id").expect("id column");
    assert_eq!(id_column.name(), "id");
    assert_eq!(id_column.data_type(), "int");
    assert!(id_column.not_null());
    assert_eq!(id_column.default(), None);
    let email_column = contacts_table.get_column("email").expect("email column");
    assert_eq!(email_column.name(), "email");
    assert_eq!(email_column.data_type(), "varchar(20)");
    assert!(!email_column.not_null());
    assert_eq!(email_column.default(), None);

    let contacts_indexes = contacts_table
        .indexes()
        .iter()
        .map(|index| index.name())
        .collect::<Vec<_>>();
    assert!(contacts_indexes[0].contains(&"PK__contacts__".to_string()));

    let users_table = schema.get("users").expect("users table");
    assert_eq!(users_table.name(), "users");
    assert_eq!(users_table.columns().len(), 2);
    let id_column = users_table.get_column("id").expect("id column");
    assert_eq!(id_column.name(), "id");
    assert_eq!(id_column.data_type(), "int");
    assert!(id_column.not_null());
    assert_eq!(id_column.default(), None);
    let email_column = users_table.get_column("email").expect("email column");
    assert_eq!(email_column.name(), "email");
    assert_eq!(email_column.data_type(), "varchar(20)");
    assert!(!email_column.not_null());
    assert_eq!(email_column.default(), None);

    let user_indexes = users_table
        .indexes()
        .iter()
        .map(|index| index.name())
        .collect::<Vec<_>>();
    assert!(user_indexes[0].contains(&"PK__users__".to_string()));

    Ok(())
}
