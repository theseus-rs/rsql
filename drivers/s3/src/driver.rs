use async_trait::async_trait;
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use file_type::FileType;
use rsql_driver::Error::IoError;
use rsql_driver::{DriverManager, Result};
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
        "s3"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let temp_dir = TempDir::new()?;
        let (file_path, file_type) = self.retrieve_file(url, temp_dir.path()).await?;
        let file_path = file_path.to_string_lossy().to_string();
        #[cfg(target_os = "windows")]
        let file_path = file_path.replace(':', "%3A").replace('\\', "/");

        debug!("temp_dir: {temp_dir:?}; file_path: {file_path}");
        let driver = DriverManager::get_by_file_type(file_type)?;
        match driver {
            Some(driver) => {
                let (_url, parameters) = url.split_once('?').unwrap_or((url, ""));
                let url = format!("{}://{file_path}?{parameters}", driver.identifier());
                let connection = driver.connect(url.as_str()).await?;
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
    async fn retrieve_file(&self, url: &str, temp_dir: &Path) -> Result<(PathBuf, &FileType)> {
        let parsed_url = Url::parse(url)?;
        let parameters: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let path = parsed_url.path().trim_start_matches('/');
        let Some(file_name) = path.split('/').last() else {
            return Err(IoError("Invalid S3 URL; no file".to_string()));
        };

        let sdk_config = aws_config::from_env().load().await;
        let mut config_builder = aws_sdk_s3::config::Builder::from(&sdk_config);

        let username = parsed_url.username();
        if !username.is_empty() {
            let password = parsed_url.password().unwrap_or_default();
            let session_token = parameters.get("session_token").cloned();
            let credentials = Credentials::from_keys(username, password, session_token);
            config_builder = config_builder.credentials_provider(credentials);
        }
        let Some(host) = parsed_url.host_str() else {
            return Err(IoError("Invalid S3 URL; no host".to_string()));
        };
        let Some((bucket, host)) = host.split_once('.') else {
            return Err(IoError(
                "Invalid S3 URL; unable to determine bucket from host".to_string(),
            ));
        };
        let Some((region, host)) = host.split_once('.') else {
            return Err(IoError(
                "Invalid S3 URL; unable to determine region from host".to_string(),
            ));
        };
        config_builder = config_builder.region(Region::new(region.to_string()));
        let port = parsed_url.port().unwrap_or(443);
        let scheme = if let Some(scheme) = parameters.get("scheme") {
            scheme
        } else if port == 80 {
            "http"
        } else {
            "https"
        };
        let endpoint_url = format!("{scheme}://{host}:{port}");
        config_builder = config_builder.endpoint_url(endpoint_url.as_str());

        let config = config_builder.build();
        let client = Client::from_conf(config);

        let mut object = client
            .get_object()
            .bucket(bucket)
            .key(path)
            .send()
            .await
            .map_err(|error| IoError(format!("Error getting object from S3: {error:?}")))?;

        create_dir_all(temp_dir)?;
        let file_path = temp_dir.join(file_name);
        let mut file = File::create_new(&file_path)
            .await
            .map_err(|error| IoError(error.to_string()))?;

        while let Some(bytes) = object.body.try_next().await.map_err(|error| {
            IoError(format!("Failed to read from S3 download stream: {error:?}"))
        })? {
            file.write_all(&bytes).await?;
        }

        let content_type = object
            .content_type
            .unwrap_or("application/octet-stream".to_string());
        let file_type = Self::file_type(content_type.as_str(), &file_path)?;
        Ok((file_path, file_type))
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
