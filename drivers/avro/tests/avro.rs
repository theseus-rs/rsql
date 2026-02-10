use rsql_driver::{Driver, Result, Value};
use rsql_driver_test_utils::dataset_url;

fn database_url() -> String {
    dataset_url("avro", "users.avro")
}

#[tokio::test(flavor = "multi_thread")]
async fn test_driver_connect() -> Result<()> {
    let database_url = database_url();
    let driver = rsql_driver_avro::Driver;
    let mut connection = driver.connect(&database_url).await?;
    assert_eq!(&database_url, connection.url());
    connection.close().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_connection_interface() -> Result<()> {
    let database_url = database_url();
    let driver = rsql_driver_avro::Driver;
    let mut connection = driver.connect(&database_url).await?;

    let mut query_result = connection
        .query("SELECT id, name FROM users ORDER BY id")
        .await?;

    assert_eq!(query_result.columns().await, vec!["id", "name"]);
    assert_eq!(
        query_result.next().await,
        Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
    );
    assert_eq!(
        query_result.next().await,
        Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
    );
    assert!(query_result.next().await.is_none());

    connection.close().await?;
    Ok(())
}
