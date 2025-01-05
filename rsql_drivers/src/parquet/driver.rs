use crate::error::Result;
use crate::polars::Connection;
use crate::url::UrlExtension;
use async_trait::async_trait;
use polars::io::SerReader;
use polars::prelude::{IntoLazy, ParquetReader};
use polars_sql::SQLContext;
use std::fs::File;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "parquet"
    }

    async fn connect(
        &self,
        url: String,
        _password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let parsed_url = Url::parse(url.as_str())?;
        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let file = File::open(&file_name)?;

        let data_frame = ParquetReader::new(file).set_rechunk(true).finish()?;

        let table_name = crate::polars::driver::get_table_name(file_name)?;
        let mut context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn file_media_type(&self) -> Option<&'static str> {
        Some("application/vnd.apache.parquet")
    }
}

#[cfg(test)]
mod test {
    use crate::test::dataset_url;
    use crate::{DriverManager, Value};

    fn database_url() -> String {
        dataset_url("parquet", "users.parquet")
    }

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let database_url = database_url();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&database_url).await?;
        assert_eq!(&database_url, connection.url());
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
