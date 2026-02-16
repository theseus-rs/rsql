use rsql_driver::{Driver, DriverManager, Result, Value};
use rsql_driver_test_utils::dataset_url;
use std::sync::Arc;

fn database_url() -> String {
    dataset_url("gzip", "users.csv.gz")
}

#[tokio::test(flavor = "multi_thread")]
async fn test_driver_connect() -> Result<()> {
    DriverManager::add(Arc::new(rsql_driver_csv::Driver))?;
    let database_url = database_url();
    let driver = rsql_driver_gzip::Driver;
    let mut connection = driver.connect(&database_url).await?;
    assert!(connection.url().contains("separator=%2C"));
    connection.close().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_connection_interface() -> Result<()> {
    DriverManager::add(Arc::new(rsql_driver_csv::Driver))?;
    let database_url = database_url();
    let driver = rsql_driver_gzip::Driver;
    let mut connection = driver.connect(&database_url).await?;

    let mut query_result = connection
        .query("SELECT id, name FROM users ORDER BY id", &[])
        .await?;

    assert_eq!(query_result.columns(), vec!["id", "name"]);
    assert_eq!(
        query_result.next().await.cloned(),
        Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
    );
    assert_eq!(
        query_result.next().await.cloned(),
        Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
    );
    assert!(query_result.next().await.is_none());

    connection.close().await?;
    Ok(())
}
