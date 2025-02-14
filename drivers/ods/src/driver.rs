use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Result;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "ods"
    }

    async fn connect(
        &self,
        url: &str,
        _password: Option<String>,
    ) -> Result<Box<dyn rsql_driver::Connection>> {
        rsql_driver_excel::Driver.connect(url, _password).await
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type
            .media_types()
            .contains(&"application/vnd.oasis.opendocument.spreadsheet")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::{Driver, Value};
    use rsql_driver_test_utils::dataset_url;

    fn database_url() -> String {
        dataset_url("ods", "users.ods")
    }

    #[tokio::test]
    async fn test_driver_connect() -> Result<()> {
        let database_url = database_url();
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url, None).await?;
        assert_eq!(&database_url, connection.url());
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> Result<()> {
        let database_url = database_url();
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url, None).await?;

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
}
