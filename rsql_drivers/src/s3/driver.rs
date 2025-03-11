use crate::DriverManager;
use async_trait::async_trait;
use aws_config::Region;
use aws_config::meta::region::RegionProviderChain;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use file_type::FileType;
use rsql_driver::Error::IoError;
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
        "s3"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let temp_dir = TempDir::new()?;
        let (file_path, file_type) = self.retrieve_file(url, temp_dir.path()).await?;
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
        let Some((bucket, key)) = path.split_once('/') else {
            return Err(IoError("Invalid S3 URL".to_string()));
        };
        let Some(file_name) = key.split('/').last() else {
            return Err(IoError("Invalid S3 URL; no file".to_string()));
        };

        let region = if let Some(region) = parameters.get("region") {
            Region::new(region.clone())
        } else {
            RegionProviderChain::default_provider()
                .region()
                .await
                .unwrap_or(Region::new("us-east-1"))
        };

        let sdk_config = aws_config::from_env().load().await;
        let mut config_builder = aws_sdk_s3::config::Builder::from(&sdk_config)
            .region(region)
            .force_path_style(true);

        let username = parsed_url.username();
        if !username.is_empty() {
            let password = parsed_url.password().unwrap_or_default();
            let session_token = parameters.get("session_token").cloned();
            let credentials = Credentials::from_keys(username, password, session_token);
            config_builder = config_builder.credentials_provider(credentials);
        }
        if let Some(host) = parsed_url.host_str() {
            let port = parsed_url.port().unwrap_or(443);
            let endpoint_url = format!("http://{host}:{port}");
            config_builder = config_builder.endpoint_url(endpoint_url.as_str());
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
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod test {
    use super::*;
    use aws_sdk_s3::primitives::ByteStream;
    use rsql_driver::{Driver, Value};
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::core::logs::LogFrame;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;
    use testcontainers_modules::{localstack::LocalStackPro, testcontainers::ImageExt};
    use tracing::info;
    use tracing_subscriber::EnvFilter;

    static ACCESS_KEY_ID: &str = "test";
    static SECRET_ACCESS_KEY: &str = "test";
    static REGION: &str = "us-east-1";

    #[tokio::test]
    async fn test_driver() -> Result<()> {
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::new("trace")
                    .add_directive("aws_config=trace".parse().expect("Invalid directive"))
                    .add_directive("aws_sdk_s3=trace".parse().expect("Invalid directive")),
            )
            .with_test_writer()
            .compact()
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);

        let container = LocalStackPro::with_auth_token(Option::<&str>::None)
            .with_env_var("SERVICES", "s3")
            .with_log_consumer(|frame: &LogFrame| {
                let mut msg =
                    std::str::from_utf8(frame.bytes()).expect("Failed to parse log message");
                if msg.ends_with('\n') {
                    msg = &msg[..msg.len() - 1];
                }
                info!("{msg}");
            })
            .start()
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let database_url = upload_test_file(&container).await?;

        let driver = crate::s3::Driver;
        let mut connection = driver.connect(database_url.as_str()).await?;

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

        connection.close().await?;
        Ok(())
    }

    async fn upload_test_file(container: &ContainerAsync<LocalStackPro>) -> Result<String> {
        let host = container
            .get_host()
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let port = container
            .get_host_port_ipv4(4566)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let bucket = "test-bucket";
        let file_name = "users.csv";
        let endpoint_url = format!("http://{host}:{port}");
        let credentials = Credentials::from_keys(ACCESS_KEY_ID, SECRET_ACCESS_KEY, None);

        let config = aws_sdk_s3::config::Builder::default()
            .region(Region::new(REGION))
            .credentials_provider(credentials)
            .endpoint_url(&endpoint_url)
            .force_path_style(true)
            .build();
        let client = Client::from_conf(config);

        client
            .create_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(|error| IoError(error.to_string()))?;

        let crate_directory = env!("CARGO_MANIFEST_DIR");
        let file_path = PathBuf::from(crate_directory)
            .join("..")
            .join("datasets")
            .join(file_name);
        let byte_stream = ByteStream::from_path(file_path)
            .await
            .map_err(|error| IoError(error.to_string()))?;

        client
            .put_object()
            .bucket(bucket)
            .key(file_name)
            .body(byte_stream)
            .send()
            .await
            .map_err(|error| IoError(error.to_string()))?;

        let database_url = format!(
            "s3://{ACCESS_KEY_ID}:{SECRET_ACCESS_KEY}@{host}:{port}/{bucket}/{file_name}?region={REGION}"
        );
        Ok(database_url)
    }
}
