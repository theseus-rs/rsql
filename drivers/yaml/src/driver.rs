use async_trait::async_trait;
use file_type::FileType;
use polars::io::SerReader;
use polars::prelude::{IntoLazy, JsonReader};
use polars_sql::SQLContext;
use rsql_driver::Error::{ConversionError, IoError};
use rsql_driver::{Result, UrlExtension};
use rsql_driver_polars::Connection;
use serde_json::json;
use std::collections::HashMap;
use std::io::Cursor;
use std::num::NonZeroUsize;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "yaml"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();

        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let json = {
            #[cfg(target_family = "wasm")]
            let yaml = std::fs::read_to_string(&file_name)?;
            #[cfg(not(target_family = "wasm"))]
            let yaml = tokio::fs::read_to_string(&file_name).await?;
            let yaml_value: serde_yaml::Value =
                serde_yaml::from_str(&yaml).map_err(|error| IoError(error.to_string()))?;
            let value = json!(yaml_value);
            serde_json::to_string(&value).map_err(|error| IoError(error.to_string()))?
        };

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

        let cursor = Cursor::new(json.as_bytes());
        let data_frame = JsonReader::new(cursor)
            .infer_schema_len(infer_schema_length)
            .set_rechunk(true)
            .with_ignore_errors(ignore_errors)
            .finish()
            .map_err(|error| IoError(error.to_string()))?;

        let table_name = rsql_driver_polars::get_table_name(file_name)?;
        let mut context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        let extensions = file_type.extensions();
        let media_types = file_type.media_types();
        media_types.contains(&"text/x-yaml")
            || media_types.contains(&"application/yaml")
            || extensions.contains(&"yml")
            || extensions.contains(&"yaml")
    }
}
