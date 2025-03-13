use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::Error::IoError;
use rsql_driver::{CachedMetadataConnection, Connection, DriverManager, Result, UrlExtension};
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "file"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn Connection>> {
        let parsed_url = Url::parse(url)?;
        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let file_type =
            FileType::try_from_file(&file_name).map_err(|error| IoError(error.to_string()))?;
        let driver = DriverManager::get_by_file_type(file_type)?;

        match driver {
            Some(driver) => {
                let scheme = format!("{}:", parsed_url.scheme());
                let url = url.strip_prefix(&scheme).unwrap_or(url);
                let url = format!("{}:{url}", driver.identifier());
                let connection = DriverManager::connect(url.as_str())
                    .await
                    .map_err(|error| IoError(error.to_string()))?;
                let connection = Box::new(CachedMetadataConnection::new(connection));
                Ok(connection)
            }
            None => Err(IoError(format!(
                "{file_name}: {:?}",
                file_type.media_types()
            ))),
        }
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}
