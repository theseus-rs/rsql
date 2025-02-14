use async_trait::async_trait;
use file_type::FileType;
use polars::io::avro::AvroReader;
use polars::io::SerReader;
use polars::prelude::IntoLazy;
use polars_sql::SQLContext;
use rsql_driver::Error::IoError;
use rsql_driver::{Result, UrlExtension};
use rsql_driver_polars::Connection;
use std::fs::File;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "avro"
    }

    async fn connect(
        &self,
        url: &str,
        _password: Option<String>,
    ) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let file = File::open(&file_name)?;

        let data_frame = AvroReader::new(file)
            .set_rechunk(true)
            .finish()
            .map_err(|error| IoError(error.to_string()))?;

        let table_name = rsql_driver_polars::get_table_name(file_name)?;
        let mut context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type
            .media_types()
            .contains(&"application/vnd.apache.avro.file")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::{Driver, Value};
    use rsql_driver_test_utils::dataset_url;

    fn database_url() -> String {
        dataset_url("avro", "users.avro")
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
