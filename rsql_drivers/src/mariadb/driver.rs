use crate::error::Result;
use crate::mysql::driver::Connection;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "mariadb"
    }

    async fn connect(
        &self,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let connection = Connection::new(url, password).await?;
        Ok(Box::new(connection))
    }
}
