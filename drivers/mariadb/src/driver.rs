use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Result;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "mariadb"
    }

    async fn connect(
        &self,
        url: &str,
        password: Option<String>,
    ) -> Result<Box<dyn rsql_driver::Connection>> {
        rsql_driver_mysql::Driver.connect(url, password).await
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}
