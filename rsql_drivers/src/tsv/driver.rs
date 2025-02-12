use crate::delimited::driver::Driver as DelimitedDriver;
use crate::error::Result;
use async_trait::async_trait;
use file_type::FileType;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "tsv"
    }

    async fn connect(
        &self,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let mut parsed_url = url::Url::parse(&url)?;
        parsed_url.query_pairs_mut().append_pair("separator", "\t");
        DelimitedDriver
            .connect(parsed_url.to_string(), password)
            .await
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type
            .media_types()
            .contains(&"text/tab-separated-values")
            || file_type.extensions().contains(&"tsv")
    }
}

#[cfg(test)]
mod test {
    use crate::test::dataset_url;
    use crate::{DriverManager, Value};

    fn database_url() -> String {
        dataset_url("tsv", "users.tsv")
    }

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let database_url = database_url();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&database_url).await?;
        assert!(connection.url().contains("separator=%09"));
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> anyhow::Result<()> {
        let database_url = database_url();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&database_url).await?;

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
