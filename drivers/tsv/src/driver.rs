use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Result;
use rsql_driver_delimited::Driver as DelimitedDriver;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "tsv"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let mut parsed_url = Url::parse(url)?;
        parsed_url.query_pairs_mut().append_pair("separator", "\t");
        DelimitedDriver.connect(parsed_url.as_str()).await
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type
            .media_types()
            .contains(&"text/tab-separated-values")
            || file_type.extensions().contains(&"tsv")
    }
}
