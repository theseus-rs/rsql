use crate::error::Result;
use crate::https;
use async_trait::async_trait;
use file_type::FileType;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "http"
    }

    async fn connect(
        &self,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        https::driver::Driver.connect(url, password).await
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        let driver = https::Driver {};
        driver.supports_file_type(file_type)
    }
}
