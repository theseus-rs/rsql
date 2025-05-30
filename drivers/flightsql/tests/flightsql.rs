#[cfg(target_os = "linux")]
use rsql_driver::{Connection, Driver, Value};
#[cfg(target_os = "linux")]
use testcontainers::runners::AsyncRunner;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_driver() -> anyhow::Result<()> {
    let image = testcontainers::ContainerRequest::from(
        testcontainers_modules::arrow_flightsql::ArrowFlightSQL::default(),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(31337).await?;
    let database_url = format!("flightsql://flight_username:test@localhost:{port}");
    let mut connection = rsql_driver_flightsql::Driver
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
        Some(vec![Value::I32(1), Value::String("foo".to_string())])
    );
    assert!(query_result.next().await.is_none());

    let metadata = connection.metadata().await?;
    assert_eq!(metadata.catalogs().len(), 3);
    let catalog = metadata.current_catalog().expect("catalog");
    assert_eq!(catalog.schemas().len(), 3);
    // TODO: Check the current schema to see if it contains the "person" table
    //let schema = catalog.current_schema().expect("schema");
    //assert!(schema.tables().iter().any(|table| table.name() == "person"));

    Ok(())
}

#[cfg(target_os = "linux")]
async fn test_data_types(connection: &mut dyn Connection) -> anyhow::Result<()> {
    let mut query_result = connection
        .query(
            "SELECT \
            NULL AS \"null\",\
            CAST(1 AS BOOL) AS bool,\
            CAST(2 AS TINYINT) AS i8,\
            CAST(3 AS SMALLINT) AS i16,\
            CAST(4 AS INTEGER) AS i32,\
            CAST(5 AS BIGINT) AS i64,\
            CAST(6.0 AS FLOAT) AS f32,\
            CAST(7.0 AS DOUBLE) AS f64,\
            'foo' AS string,
            CAST('baz' AS BINARY) AS bytes,\
            CAST('2001-12-31' AS DATE) AS date,\
            CAST('12:34:56' AS TIME) AS time,\
            CAST('2001-12-31 12:34:56' AS TIMESTAMP) AS timestamp
        ",
        )
        .await?;

    let columns = query_result.columns().await;
    assert_eq!(
        columns,
        vec![
            "null",
            "bool",
            "i8",
            "i16",
            "i32",
            "i64",
            "f32",
            "f64",
            "string",
            "bytes",
            "date",
            "time",
            "timestamp"
        ]
    );

    let row = query_result.next().await.expect("no row");
    let values = row.to_vec();
    assert_eq!(
        values,
        vec![
            Value::Null,
            Value::Bool(true),
            Value::I8(2),
            Value::I16(3),
            Value::I32(4),
            Value::I64(5),
            Value::F32(6.0),
            Value::F64(7.0),
            Value::String("foo".to_string()),
            Value::Bytes(vec![98, 97, 122]),
            Value::Date(jiff::civil::Date::constant(2001, 12, 31)),
            Value::Time(jiff::civil::Time::constant(12, 34, 56, 0)),
            Value::DateTime(jiff::civil::DateTime::constant(2001, 12, 31, 12, 34, 56, 0)),
        ]
    );
    assert!(query_result.next().await.is_none());
    Ok(())
}
