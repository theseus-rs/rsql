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

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        rsql_driver_excel::Driver.connect(url).await
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type
            .media_types()
            .contains(&"application/vnd.oasis.opendocument.spreadsheet")
    }
}
