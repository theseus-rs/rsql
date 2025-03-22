use async_trait::async_trait;
use file_type::FileType;
use polars::io::SerReader;
use polars::io::avro::AvroReader;
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

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
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
        file_type.extensions().contains(&"avro")
    }
}
