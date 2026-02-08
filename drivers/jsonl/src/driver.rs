use async_trait::async_trait;
use file_type::FileType;
use polars::io::SerReader;
use polars::io::json::{JsonFormat, JsonReader};
use polars::prelude::IntoLazy;
use polars_sql::SQLContext;
use rsql_driver::Error::{ConversionError, IoError};
use rsql_driver::{Result, UrlExtension};
use rsql_driver_polars::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::num::NonZeroUsize;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "jsonl"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();

        // Read Options
        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let file = File::open(&file_name)?;
        let ignore_errors = query_parameters
            .get("ignore_errors")
            .is_some_and(|value| value == "true");
        let infer_schema_length = match query_parameters.get("infer_schema_length") {
            Some(infer_schema_length) => {
                let length = infer_schema_length
                    .parse::<usize>()
                    .map_err(|error| ConversionError(error.to_string()))?;
                if length == 0 {
                    None
                } else {
                    NonZeroUsize::new(length)
                }
            }
            None => NonZeroUsize::new(100),
        };

        let data_frame = JsonReader::new(file)
            .with_json_format(JsonFormat::JsonLines)
            .infer_schema_len(infer_schema_length)
            .set_rechunk(true)
            .with_ignore_errors(ignore_errors)
            .finish()
            .map_err(|error: polars::prelude::PolarsError| IoError(error.to_string()))?;

        let table_name = rsql_driver_polars::get_table_name(file_name)?;
        let context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type.media_types().contains(&"application/jsonl")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::{Driver, Value};
    use rsql_driver_test_utils::dataset_url;

    fn database_url() -> String {
        dataset_url("jsonl", "users.jsonl")
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_driver_connect() -> Result<()> {
        let database_url = database_url();
        let driver_manager = crate::Driver;
        let mut connection = driver_manager.connect(&database_url).await?;
        assert_eq!(&database_url, connection.url());
        connection.close().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_connection_interface() -> Result<()> {
        let database_url = database_url();
        let driver_manager = crate::Driver;
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
