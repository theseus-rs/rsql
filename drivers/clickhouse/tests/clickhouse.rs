#[cfg(target_os = "linux")]
use jiff::civil::{Date, DateTime};
#[cfg(target_os = "linux")]
use rsql_driver::{Driver, Value};
#[cfg(target_os = "linux")]
use testcontainers::runners::AsyncRunner;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_clickhouse_driver() -> anyhow::Result<()> {
    let image = testcontainers::ContainerRequest::from(
        testcontainers_modules::clickhouse::ClickHouse::default(),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(8123).await?;

    let database_url = format!("clickhouse://localhost:{port}?scheme=http");
    let driver = rsql_driver_clickhouse::Driver;
    let mut connection = driver.connect(database_url.as_str()).await?;
    assert_eq!(database_url, connection.url().as_str());

    let mut query_result = connection.query("SELECT 1").await?;
    let row = query_result.next().await.expect("no row");
    let value = row.first().expect("no value");

    assert_eq!(*value, Value::U8(1));

    container.stop().await?;
    container.rm().await?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_clickhouse_types() -> anyhow::Result<()> {
    let image = testcontainers::ContainerRequest::from(
        testcontainers_modules::clickhouse::ClickHouse::default(),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(8123).await?;

    let database_url = format!("clickhouse://localhost:{port}?scheme=http");
    let driver = rsql_driver_clickhouse::Driver;
    let mut connection = driver.connect(database_url.as_str()).await?;

    let test_cases = vec![
        ("SELECT NULL", Value::Null),
        ("SELECT nullIf(1, 1)", Value::Null),
        ("SELECT true", Value::Bool(true)),
        ("SELECT false", Value::Bool(false)),
        ("SELECT toInt8(127)", Value::I8(i8::MAX)),
        ("SELECT toInt16(32767)", Value::I16(i16::MAX)),
        ("SELECT toInt32(2147483647)", Value::I32(i32::MAX)),
        ("SELECT toInt64(9223372036854775807)", Value::I64(i64::MAX)),
        (
            "SELECT toInt128(9223372036854775807)",
            Value::I128(i64::MAX as i128),
        ),
        ("SELECT toUInt8(255)", Value::U8(u8::MAX)),
        ("SELECT toUInt16(65535)", Value::U16(u16::MAX)),
        ("SELECT toUInt32(4294967295)", Value::U32(u32::MAX)),
        (
            "SELECT toUInt64(18446744073709551615)",
            Value::U64(u64::MAX),
        ),
        (
            "SELECT toUInt128(18446744073709551615)",
            Value::U128(u64::MAX as u128),
        ),
        ("SELECT toFloat32(1.23)", Value::F32(1.23)),
        ("SELECT toFloat64(1.23)", Value::F64(1.23)),
        ("SELECT 1.23", Value::F64(1.23)),
        (
            "SELECT 'Hello World'",
            Value::String("Hello World".to_string()),
        ),
        ("SELECT toString(123)", Value::String("123".to_string())),
        (
            "SELECT 'ClickHouse Test'",
            Value::String("ClickHouse Test".to_string()),
        ),
        ("SELECT ''", Value::String("".to_string())),
        (
            "SELECT toDate('2023-12-25')",
            Value::Date(Date::constant(2023, 12, 25)),
        ),
        (
            "SELECT toDateTime('2023-12-25 15:30:00')",
            Value::DateTime(DateTime::constant(2023, 12, 25, 15, 30, 0, 0)),
        ),
        (
            "SELECT [1, 2, 3]",
            Value::Array(vec![Value::U8(1), Value::U8(2), Value::U8(3)]),
        ),
        (
            "SELECT ['a', 'b', 'c']",
            Value::Array(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string()),
                Value::String("c".to_string()),
            ]),
        ),
        ("SELECT []", Value::Array(Vec::new())),
        (
            "SELECT [1, 2, NULL]",
            Value::Array(vec![Value::U8(1), Value::U8(2), Value::Null]),
        ),
    ];

    for (query, expected) in test_cases {
        let mut query_result = connection.query(query).await?;
        let row = query_result.next().await.expect("no row");
        let value = row.first().expect("no value");
        assert_eq!(*value, expected, "Failed for query: {query}");
    }

    container.stop().await?;
    container.rm().await?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_clickhouse_multiple_columns() -> anyhow::Result<()> {
    let image = testcontainers::ContainerRequest::from(
        testcontainers_modules::clickhouse::ClickHouse::default(),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(8123).await?;

    let database_url = format!("clickhouse://localhost:{port}?scheme=http");
    let driver = rsql_driver_clickhouse::Driver;
    let mut connection = driver.connect(database_url.as_str()).await?;

    let mut query_result = connection
        .query("SELECT 42, 'hello', true, NULL, 1.23")
        .await?;
    let row = query_result.next().await.expect("no row");

    assert_eq!(row.len(), 5);
    assert_eq!(*row.first().expect("value"), Value::U8(42));
    assert_eq!(
        *row.get(1).expect("value"),
        Value::String("hello".to_string())
    );
    assert_eq!(*row.get(2).expect("value"), Value::Bool(true));
    assert_eq!(*row.get(3).expect("value"), Value::Null);
    match row.get(4).expect("value") {
        Value::F64(f) => assert!((f - 1.23).abs() < 0.0001),
        _ => panic!("Expected F64 value"),
    }

    container.stop().await?;
    container.rm().await?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_clickhouse_dynamic_queries() -> anyhow::Result<()> {
    let image = testcontainers::ContainerRequest::from(
        testcontainers_modules::clickhouse::ClickHouse::default(),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(8123).await?;

    let database_url = format!("clickhouse://localhost:{port}?scheme=http");
    let driver = rsql_driver_clickhouse::Driver;
    let mut connection = driver.connect(database_url.as_str()).await?;

    connection
        .execute("CREATE TABLE test_table (id UInt32, name String, value Float64) ENGINE = Memory")
        .await?;
    connection
        .execute("INSERT INTO test_table VALUES (1, 'test1', 1.5), (2, 'test2', 2.5)")
        .await?;

    let mut query_result = connection
        .query("SELECT * FROM test_table ORDER BY id")
        .await?;

    let row1 = query_result.next().await.expect("no row 1");
    assert_eq!(row1.len(), 3);
    assert_eq!(*row1.first().expect("value"), Value::U32(1));
    assert_eq!(
        *row1.get(1).expect("value"),
        Value::String("test1".to_string())
    );
    match row1.get(2).expect("value") {
        Value::F64(f) => assert!((f - 1.5).abs() < 0.0001),
        _ => panic!("Expected F64 value"),
    }

    let row2 = query_result.next().await.expect("no row 2");
    assert_eq!(row2.len(), 3);
    assert_eq!(*row2.first().expect("value"), Value::U32(2));
    assert_eq!(
        *row2.get(1).expect("value"),
        Value::String("test2".to_string())
    );
    match row2.get(2).expect("value") {
        Value::F64(f) => assert!((f - 2.5).abs() < 0.0001),
        _ => panic!("Expected F64 value"),
    }

    let mut agg_result = connection
        .query("SELECT COUNT(*), SUM(value) FROM test_table")
        .await?;
    let agg_row = agg_result.next().await.expect("no aggregation row");
    assert_eq!(*agg_row.first().expect("value"), Value::U64(2));
    match agg_row.get(1).expect("value") {
        Value::F64(f) => assert!((f - 4.0).abs() < 0.0001),
        _ => panic!("Expected F64 value for sum"),
    }

    connection.execute("DROP TABLE test_table").await?;

    container.stop().await?;
    container.rm().await?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_clickhouse_metadata() -> anyhow::Result<()> {
    let image = testcontainers::ContainerRequest::from(
        testcontainers_modules::clickhouse::ClickHouse::default(),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(8123).await?;

    let database_url = format!("clickhouse://localhost:{port}?scheme=http");
    let driver = rsql_driver_clickhouse::Driver;
    let mut connection = driver.connect(database_url.as_str()).await?;

    connection.execute("CREATE DATABASE test_metadata").await?;

    let database_url_with_db = format!("clickhouse://localhost:{port}/test_metadata?scheme=http");
    let mut connection = driver.connect(database_url_with_db.as_str()).await?;

    connection
        .execute(
            "CREATE TABLE users (
            id UInt32,
            name String,
            email String,
            age UInt8,
            balance Float64,
            created_at DateTime,
            is_active Bool,
            tags Array(String)
        ) ENGINE = MergeTree()
        ORDER BY id
        PRIMARY KEY id",
        )
        .await?;

    connection
        .execute(
            "CREATE TABLE orders (
            order_id UInt64,
            user_id UInt32,
            amount Decimal(10,2),
            status String,
            order_date Date
        ) ENGINE = MergeTree()
        ORDER BY (user_id, order_date)",
        )
        .await?;

    connection
        .execute(
            "CREATE VIEW user_orders AS
        SELECT u.name, o.amount, o.order_date
        FROM users u
        JOIN orders o ON u.id = o.user_id",
        )
        .await?;

    let metadata = connection.metadata().await?;
    let catalogs = metadata.catalogs();
    let test_db = catalogs
        .iter()
        .find(|catalog| catalog.name() == "test_metadata")
        .expect("test_metadata database not found");

    assert_eq!(test_db.schemas().len(), 1);
    let default_schema = &test_db.schemas()[0];
    assert_eq!(default_schema.name(), "default");

    let tables = default_schema.tables();
    assert!(tables.len() >= 3);

    let users_table = tables
        .iter()
        .find(|t| t.name() == "users")
        .expect("users table not found");

    assert_eq!(users_table.name(), "users");
    assert_eq!(users_table.columns().len(), 8);

    let expected_columns = [
        ("id", "UInt32"),
        ("name", "String"),
        ("email", "String"),
        ("age", "UInt8"),
        ("balance", "Float64"),
        ("created_at", "DateTime"),
        ("is_active", "Bool"),
        ("tags", "Array(String)"),
    ];

    let user_columns = users_table.columns();
    for (expected_name, expected_type) in expected_columns {
        let column = user_columns
            .iter()
            .find(|c| c.name() == expected_name)
            .unwrap_or_else(|| panic!("Column {expected_name} not found"));

        assert_eq!(column.name(), expected_name);
        assert!(
            column.data_type().contains(expected_type) || column.data_type() == expected_type,
            "Expected type '{expected_type}', got '{}'",
            column.data_type()
        );
    }

    let user_indexes = users_table.indexes();
    if !user_indexes.is_empty() {
        let primary_index = user_indexes
            .iter()
            .find(|i| i.name() == "PRIMARY")
            .expect("PRIMARY index not found");

        assert_eq!(primary_index.name(), "PRIMARY");
        assert!(primary_index.unique());
        assert_eq!(primary_index.columns(), &["id"]);
    }

    let orders_table = tables
        .iter()
        .find(|t| t.name() == "orders")
        .expect("orders table not found");

    assert_eq!(orders_table.name(), "orders");
    assert_eq!(orders_table.columns().len(), 5);

    let order_columns = orders_table.columns();
    let order_column_names: Vec<&str> = order_columns.iter().map(|c| c.name()).collect();

    assert!(order_column_names.contains(&"order_id"));
    assert!(order_column_names.contains(&"user_id"));
    assert!(order_column_names.contains(&"amount"));
    assert!(order_column_names.contains(&"status"));
    assert!(order_column_names.contains(&"order_date"));

    let user_orders_view = tables
        .iter()
        .find(|t| t.name() == "user_orders")
        .expect("user_orders view not found");

    assert_eq!(user_orders_view.name(), "user_orders");

    let system_db_exists = catalogs.iter().any(|catalog| catalog.name() == "system");
    assert!(system_db_exists, "system database should exist");

    let default_db_exists = catalogs.iter().any(|catalog| catalog.name() == "default");
    assert!(default_db_exists, "default database should exist");

    let database_url = format!("clickhouse://localhost:{port}?scheme=http");
    let mut connection = driver.connect(database_url.as_str()).await?;
    connection.execute("DROP DATABASE test_metadata").await?;

    container.stop().await?;
    container.rm().await?;
    Ok(())
}
