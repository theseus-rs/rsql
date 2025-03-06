use crate::DriverManager;
use async_trait::async_trait;
use file_type::FileType;
use futures_util::StreamExt;
use reqwest::header::HeaderMap;
use rsql_driver::Error::{ConversionError, IoError};
use rsql_driver::Result;
use std::collections::HashMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "https"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let temp_dir = TempDir::new()?;
        let (request_headers, file_path, file_type, response_headers) =
            self.retrieve_file(url, temp_dir.path()).await?;
        let file_path = file_path.to_string_lossy().to_string();
        #[cfg(target_os = "windows")]
        let file_path = file_path.replace(':', "%3A").replace('\\', "/");

        debug!("temp_dir: {temp_dir:?}; file_path: {file_path}");
        let driver_manager = DriverManager::default();
        let driver = driver_manager.get_by_file_type(file_type);
        match driver {
            Some(driver) => {
                let (_url, parameters) = url.split_once('?').unwrap_or((url, ""));
                let url = format!("{}://{file_path}?{parameters}", driver.identifier());
                let mut connection = driver.connect(url.as_str()).await?;
                create_header_tables(&mut connection, &request_headers, &response_headers).await?;
                Ok(connection)
            }
            None => Err(IoError(format!(
                "{file_path:?}: {:?}",
                file_type.media_types()
            ))),
        }
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

impl Driver {
    async fn retrieve_file(
        &self,
        url: &str,
        temp_dir: &Path,
    ) -> Result<(
        HashMap<String, String>,
        PathBuf,
        &FileType,
        HashMap<String, String>,
    )> {
        let mut parsed_url = Url::parse(url)?;
        let file_path = PathBuf::from(parsed_url.path());
        // Extract the last segment of the path as a file name
        let file_name = match file_path.file_name() {
            Some(file_name) => file_name.to_string_lossy().to_string(),
            None => "response".to_string(),
        };

        let mut request_headers: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();
        if let Some(headers) = request_headers.remove("_headers") {
            // Split individual headers by ; with key=value pairs
            let headers = headers
                .split(';')
                .map(|header| {
                    let mut parts = header.split('=');
                    let key = parts.next().unwrap_or_default().to_string();
                    let value = parts.next().unwrap_or_default().to_string();
                    (key, value)
                })
                .collect::<HashMap<String, String>>();
            request_headers.extend(headers);
        }

        parsed_url.set_query(None);
        let url = parsed_url.to_string();
        let parameters: HashMap<&str, &str> = request_headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let parsed_url = Url::parse_with_params(url.as_str(), parameters)?;

        if !request_headers
            .keys()
            .any(|key| key.eq_ignore_ascii_case("user-agent"))
        {
            let version: &str = env!("CARGO_PKG_VERSION");
            let os = std::env::consts::OS;
            let arch = std::env::consts::ARCH;
            let user_agent = format!("rsql/{version} ({os}; {arch})");
            request_headers.insert("User-Agent".to_string(), user_agent);
        }

        let header_map: HeaderMap = (&request_headers)
            .try_into()
            .map_err(|_| ConversionError("MalformedHeaders".into()))?;
        let client = reqwest::ClientBuilder::new()
            .default_headers(header_map)
            .build()
            .map_err(|error| IoError(error.to_string()))?;

        let response = client
            .get(parsed_url.as_str())
            .send()
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let response_headers = response.headers();
        let response_headers: HashMap<String, String> = response_headers
            .iter()
            .map(|(key, value)| {
                (
                    key.as_str().to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect();
        let content_type = response_headers
            .iter()
            .find(|(key, _value)| key.eq_ignore_ascii_case("content-type"))
            .map(|(_key, value)| value.split(';').next().unwrap_or_default())
            .unwrap_or_default();
        create_dir_all(temp_dir)?;
        let file_path = temp_dir.join(file_name);
        let mut file = File::create_new(&file_path)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let mut stream = response.bytes_stream();
        while let Some(item) = stream.next().await {
            let item = item.map_err(|error| IoError(error.to_string()))?;
            file.write_all(&item)
                .await
                .map_err(|error| IoError(error.to_string()))?;
        }

        let file_type = Self::file_type(content_type, &file_path)?;
        Ok((request_headers, file_path, file_type, response_headers))
    }

    fn file_type(content_type: &str, file_path: &PathBuf) -> Result<&'static FileType> {
        // Ignore generic content types and try to determine the file type from the extension
        // or bytes
        let content_type = content_type.trim().to_lowercase();
        if !["text/plain", "application/octet-stream"].contains(&content_type.as_str()) {
            let file_types = FileType::from_media_type(content_type.to_lowercase());
            if !file_types.is_empty() {
                if let Some(file_type) = file_types.first() {
                    return Ok(file_type);
                }
            }
        }
        let file_type =
            FileType::try_from_file(file_path).map_err(|error| IoError(error.to_string()))?;
        Ok(file_type)
    }
}

async fn create_header_tables(
    connection: &mut Box<dyn rsql_driver::Connection>,
    request_headers: &HashMap<String, String>,
    response_headers: &HashMap<String, String>,
) -> Result<()> {
    let request_header_sql = create_table_sql("request_headers", request_headers);
    connection.execute(&request_header_sql).await?;
    let response_header_sql = create_table_sql("response_headers", response_headers);
    connection.execute(&response_header_sql).await?;
    Ok(())
}

fn create_table_sql(table_name: &str, headers: &HashMap<String, String>) -> String {
    let columns = headers
        .iter()
        .map(|(key, value)| {
            let key = key.replace('\'', "''").to_lowercase();
            let value = value.replace('\'', "''");
            format!("SELECT '{key}' AS \"header\", '{value}' AS \"value\"")
        })
        .collect::<Vec<String>>()
        .join(" UNION ");
    format!("CREATE TABLE {table_name} AS {columns}")
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::{Driver, Value};

    #[tokio::test]
    async fn test_driver() -> Result<()> {
        let database_url =
            "https://raw.githubusercontent.com/theseus-rs/rsql/refs/heads/main/datasets/users.csv";
        let driver = crate::https::Driver;
        let mut connection = driver.connect(database_url).await?;

        let mut query_result = connection
            .query("SELECT id, name FROM users ORDER BY id")
            .await?;

        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
        );
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
        );
        assert!(query_result.next().await.is_none());

        let mut query_result = connection
            .query("SELECT value FROM request_headers WHERE header = 'user-agent'")
            .await?;
        let row = query_result.next().await.expect("row");
        let value = row[0].to_string();
        assert!(value.contains("rsql"));

        let mut query_result = connection
            .query("SELECT value FROM response_headers WHERE header = 'content-type'")
            .await?;
        let row = query_result.next().await.expect("row");
        let value = row[0].to_string();
        assert!(value.contains("text/plain"));

        connection.close().await?;
        Ok(())
    }
}
