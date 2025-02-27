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
        let driver = crate::https::Driver;
        driver.connect(url).await
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        let driver = crate::https::Driver;
        driver.supports_file_type(file_type)
    }
}
