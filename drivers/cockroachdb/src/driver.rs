use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Result;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "cockroachdb"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let driver = rsql_driver_postgresql::Driver;
        driver.connect(url).await
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}
