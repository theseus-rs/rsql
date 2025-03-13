use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Result;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "http"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let driver = rsql_driver_https::Driver;
        driver.connect(url).await
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        let driver = rsql_driver_https::Driver;
        driver.supports_file_type(file_type)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::{Driver, DriverManager, Value};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_driver() -> Result<()> {
        DriverManager::add(Arc::new(rsql_driver_csv::Driver))?;
        let database_url =
            "http://raw.githubusercontent.com/theseus-rs/rsql/refs/heads/main/datasets/users.csv";
        let driver = Driver;
        let mut connection = driver.connect(database_url).await?;

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

        let mut query_result = connection
            .query("SELECT value FROM request_headers WHERE header = 'user-agent'")
            .await?;
        let row = query_result.next().await.expect("row");
        let value = row[0].to_string();
        assert!(value.contains("rsql"));

        let mut query_result = connection
            .query("SELECT value FROM response_headers WHERE header = 'content-type'")
            .await?;
        let row = query_result.next().await.expect("row");
        let value = row[0].to_string();
        assert!(value.contains("text/plain"));

        connection.close().await?;
        Ok(())
    }
}
