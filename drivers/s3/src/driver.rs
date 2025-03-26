use async_trait::async_trait;
use aws_config::{AppName, Region};
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use file_type::FileType;
use rsql_driver::Error::IoError;
use rsql_driver::{DriverManager, Result};
use std::collections::HashMap;
use std::env;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use url::Url;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

/// Driver that connects to a Simple Storage Service (S3).
///
/// For a list of supported environment variables, see:
/// <https://docs.aws.amazon.com/sdkref/latest/guide/settings-reference.html#EVarSettings>
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
    /// Retrieves a file from an S3 bucket and stores it in a temporary directory.
    async fn retrieve_file(&self, url: &str, temp_dir: &Path) -> Result<(PathBuf, &FileType)> {
        let parsed_url = Url::parse(url)?;
        let parameters: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
        let use_endpoint_url = parameters.contains_key("scheme");
        let path = parsed_url.path().trim_start_matches('/');
        let (bucket, key) = if use_endpoint_url {
            // If the URL is a path style, use the first part of the path as the bucket
            // (e.g. s3://host:port/bucket/key)
            let Some((bucket, key)) = path.split_once('/') else {
                return Err(IoError("Invalid S3 URL; no bucket".to_string()));
            };
            (bucket, key)
        } else {
            // If the URL is a S3 URI, use the host as the bucket (e.g. s3://bucket/key)
            let Some(host) = parsed_url.host_str() else {
                return Err(IoError("Invalid S3 URL; no bucket (host) ".to_string()));
            };
            (host, path)
        };
        let Some(file_name) = key.split('/').last() else {
            return Err(IoError("Invalid S3 URL; no file".to_string()));
        };

        let sdk_config = aws_config::from_env().load().await;
        let mut config_builder = aws_sdk_s3::config::Builder::from(&sdk_config);

        if let Ok(app_name) = AppName::new(PACKAGE_NAME) {
            config_builder = config_builder.app_name(app_name);
        }
        if let Some(credentials) = Self::credentials(&parsed_url, &parameters) {
            config_builder = config_builder.credentials_provider(credentials);
        }
        if let Some(region) = Self::region(&parameters) {
            config_builder = config_builder.region(region);
        }
        if use_endpoint_url {
            let Some(endpoint_url) = Self::endpoint_url(&parsed_url, &parameters) else {
                return Err(IoError(
                    "Invalid S3 URL; no endpoint url defined".to_string(),
                ));
            };
            config_builder = config_builder.endpoint_url(endpoint_url.as_str());
        }
        if let Some(force_path_style) = parameters.get("force_path_style") {
            let force_path_style = force_path_style.parse::<bool>().unwrap_or(false);
            config_builder = config_builder.force_path_style(force_path_style);
        }

        let config = config_builder.build();
        let client = Client::from_conf(config);
        let mut object = client
            .get_object()
            .bucket(bucket)
            .key(key)
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

    /// Extracts the credentials from the URL and returns them as a `Credentials` object.
    /// If the URL does not contain credentials, it will look for S3 specific environment variables.
    fn credentials(parsed_url: &Url, parameters: &HashMap<String, String>) -> Option<Credentials> {
        let username = parsed_url.username();
        if username.is_empty() {
            return None;
        }

        let access_key = username.to_string();
        let secret_key = parsed_url.password()?.to_string();
        let session_token = parameters.get("session_token").cloned();
        Some(Credentials::from_keys(
            access_key,
            secret_key,
            session_token,
        ))
    }

    /// Extracts the region from the URL, or the `S3_REGION` environment variable and returns it as
    /// a `Region` object.
    fn region(parameters: &HashMap<String, String>) -> Option<Region> {
        parameters
            .get("region")
            .map(|region| Region::new(region.to_string()))
    }

    /// Extracts the endpoint URL from the URL and returns it as a string.
    fn endpoint_url(parsed_url: &Url, parameters: &HashMap<String, String>) -> Option<String> {
        if let Some(host) = parsed_url.host_str() {
            let port = parsed_url.port().unwrap_or(443);
            let scheme = parameters
                .get("scheme")
                .cloned()
                .unwrap_or("https".to_string());
            let endpoint_url = format!("{scheme}://{host}:{port}");
            Some(endpoint_url)
        } else {
            None
        }
    }
}
