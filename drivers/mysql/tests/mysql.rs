#[cfg(target_os = "linux")]
use indoc::indoc;
#[cfg(target_os = "linux")]
use jiff::civil;
#[cfg(target_os = "linux")]
use rsql_driver::{Connection, Driver, Value};
#[cfg(target_os = "linux")]
use serde_json::json;
#[cfg(target_os = "linux")]
use std::str::FromStr;
#[cfg(target_os = "linux")]
use testcontainers::runners::AsyncRunner;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_mysql_driver() -> anyhow::Result<()> {
    let image =
        testcontainers::ContainerRequest::from(testcontainers_modules::mysql::Mysql::default());
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(3306).await?;

    let database_url = format!("mysql://root@127.0.0.1:{port}/mysql");
    let mut connection = rsql_driver_mysql::Driver
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
        Some(vec![Value::I16(1), Value::String("foo".to_string())])
    );
    assert!(query_result.next().await.is_none());

    let metadata = connection.metadata().await?;
    let catalog = metadata.current_catalog().expect("catalog");
    let schema = catalog.current_schema().expect("schema");
    assert!(schema.tables().iter().any(|table| table.name() == "person"));

    Ok(())
}

#[cfg(target_os = "linux")]
async fn test_data_types(connection: &mut dyn Connection) -> anyhow::Result<()> {
    let sql = indoc! {r"
            CREATE TABLE data_types (
                char_type CHAR,
                varchar_type VARCHAR(50),
                text_type TEXT,
                binary_type BINARY(3),
                varbinary_type VARBINARY(50),
                blob_type BLOB,
                tinyint_type TINYINT,
                smallint_type SMALLINT,
                mediumint_type MEDIUMINT,
                int_type INT,
                bigint_type BIGINT,
                bigint_unsigned_type BIGINT UNSIGNED,
                decimal_type DECIMAL(5,2),
                float_type FLOAT,
                double_type DOUBLE,
                date_type DATE,
                time_type TIME,
                datetime_type DATETIME,
                timestamp_type TIMESTAMP,
                json_type JSON
            )
        "};
    let _ = connection.execute(sql).await?;

    let sql = indoc! {r#"
            INSERT INTO data_types (
                char_type, varchar_type, text_type, binary_type, varbinary_type, blob_type,
                tinyint_type, smallint_type, mediumint_type, int_type, bigint_type,
                bigint_unsigned_type, decimal_type, float_type, double_type, date_type, time_type,
                datetime_type, timestamp_type, json_type
            ) VALUES (
                 'a', 'foo', 'foo', 'foo', 'foo', 'foo',
                 127, 32767, 8388607, 2147483647, 9223372036854775807, 18446744073709551615,
                 123.45, 123.0, 123.0, '2022-01-01', '14:30:00', '2022-01-01 14:30:00',
                 '2022-01-01 14:30:00', '{"key": "value"}'
             )
        "#};
    let _ = connection.execute(sql).await?;

    let sql = indoc! {r"
            SELECT char_type, varchar_type, text_type, binary_type, varbinary_type, blob_type,
                   tinyint_type, smallint_type, mediumint_type, int_type, bigint_type,
                   bigint_unsigned_type, decimal_type, float_type, double_type, date_type,
                   time_type, datetime_type, timestamp_type, json_type
              FROM data_types
        "};
    let mut query_result = connection.query(sql).await?;
    assert_eq!(
        query_result.next().await,
        Some(vec![
            Value::String("a".to_string()),
            Value::String("foo".to_string()),
            Value::String("foo".to_string()),
            Value::Bytes("foo".as_bytes().to_vec()),
            Value::Bytes("foo".as_bytes().to_vec()),
            Value::Bytes("foo".as_bytes().to_vec()),
            Value::I16(127),
            Value::I16(32_767),
            Value::I32(8_388_607),
            Value::I32(2_147_483_647),
            Value::I64(9_223_372_036_854_775_807),
            Value::U64(18_446_744_073_709_551_615),
            Value::Decimal(rust_decimal::Decimal::from_str("123.45").expect("invalid decimal")),
            Value::F32(123.0),
            Value::F32(123.0),
            Value::Date(civil::date(2022, 1, 1)),
            Value::Time(civil::time(14, 30, 0, 0)),
            Value::DateTime(civil::datetime(2022, 1, 1, 14, 30, 0, 0)),
            Value::DateTime(civil::datetime(2022, 1, 1, 14, 30, 0, 0)),
            Value::from(json!({"key": "value"}))
        ])
    );
    assert!(query_result.next().await.is_none());

    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_mysql_metadata() -> anyhow::Result<()> {
    let image =
        testcontainers::ContainerRequest::from(testcontainers_modules::mysql::Mysql::default());
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(3306).await?;

    let database_url = &format!("mysql://root@127.0.0.1:{port}/mysql");
    let mut connection = rsql_driver_mysql::Driver
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
    let catalog = metadata.current_catalog().expect("catalog");
    let schema = catalog.current_schema().expect("schema");

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

    assert_eq!(contacts_table.indexes().len(), 1);
    let primary_key_index = contacts_table.get_index("PRIMARY").expect("index");
    assert_eq!(primary_key_index.name(), "PRIMARY");
    assert_eq!(primary_key_index.columns(), ["id"]);
    assert!(primary_key_index.unique());

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

    assert_eq!(users_table.indexes().len(), 1);
    let primary_key_index = users_table.get_index("PRIMARY").expect("index");
    assert_eq!(primary_key_index.name(), "PRIMARY");
    assert_eq!(primary_key_index.columns(), ["id"]);
    assert!(primary_key_index.unique());

    Ok(())
}
