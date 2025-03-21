use rsql_driver::{Driver, Result, Value};
use rsql_driver_test_utils::dataset_url;

fn database_url() -> String {
    dataset_url("fwf", "users.fwf")
}

#[tokio::test]
async fn test_driver_connect() -> Result<()> {
    let database_url = format!("{}?widths=4,15", database_url());
    let driver = rsql_driver_fwf::Driver;
    let mut connection = driver.connect(&database_url).await?;
    assert_eq!(&database_url, connection.url());
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_connection_interface() -> Result<()> {
    let database_url = format!("{}?widths=4,15&headers=id,name", database_url());
    let driver = rsql_driver_fwf::Driver;
    let mut connection = driver.connect(&database_url).await?;

    let mut query_result = connection
        .query("SELECT id, name FROM users ORDER BY id")
        .await?;

    assert_eq!(query_result.columns().await, vec!["id", "name"]);
    assert_eq!(
        query_result.next().await,
        Some(vec![
            Value::String("1".to_string()),
            Value::String("John Doe".to_string())
        ])
    );
    assert_eq!(
        query_result.next().await,
        Some(vec![
            Value::String("2".to_string()),
            Value::String("Jane Smith".to_string())
        ])
    );
    assert!(query_result.next().await.is_none());

    connection.close().await?;
    Ok(())
}
