use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Result;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "jdbc"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        Ok(Box::new(
            crate::Connection::new(url, crate::Options::parse(url)?).await?,
        ))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}
