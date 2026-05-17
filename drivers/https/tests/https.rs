use rsql_driver::{Driver, DriverManager, Result, Value};
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const USERS_CSV: &str = "id,name\n1,John Doe\n2,Jane Smith\n";

async fn mock_database_url() -> (MockServer, String) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/users.csv"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/csv")
                .set_body_string(USERS_CSV),
        )
        .mount(&server)
        .await;
    let database_url = format!("{}/users.csv", server.uri());
    (server, database_url)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_driver_connect() -> Result<()> {
    DriverManager::add(Arc::new(rsql_driver_csv::Driver))?;
    let (_server, database_url) = mock_database_url().await;
    let driver = rsql_driver_https::Driver;
    let mut connection = driver.connect(&database_url).await?;
    assert!(connection.url().contains("separator=%2C"));
    connection.close().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_connection_interface() -> Result<()> {
    DriverManager::add(Arc::new(rsql_driver_csv::Driver))?;
    let (_server, database_url) = mock_database_url().await;
    let driver = rsql_driver_https::Driver;
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
