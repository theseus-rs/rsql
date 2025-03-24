use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use rsql_driver::Error::IoError;
use rsql_driver::{Driver, DriverManager, Result, Value};
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::Arc;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::core::Host;
use testcontainers_modules::testcontainers::core::logs::LogFrame;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::{localstack::LocalStackPro, testcontainers::ImageExt};
use tracing::info;
use tracing_subscriber::EnvFilter;

static HOST: &str = "s3.localhost.localstack.cloud";
static BUCKET: &str = "test-bucket";
static ACCESS_KEY_ID: &str = "test";
static SECRET_ACCESS_KEY: &str = "test";
static REGION: &str = "us-east-1";

#[tokio::test]
async fn test_driver() -> Result<()> {
    if env::var("CI").unwrap_or_default() == "true" && env::consts::OS != "linux" {
        eprintln!("Skipping CI test on non-linux platform");
        return Ok(());
    }

    DriverManager::add(Arc::new(rsql_driver_csv::Driver))?;
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

    let host = Host::Addr(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let container = LocalStackPro::with_auth_token(Option::<&str>::None)
        .with_env_var("SERVICES", "s3")
        .with_host(HOST, host)
        .with_log_consumer(|frame: &LogFrame| {
            let mut msg = std::str::from_utf8(frame.bytes()).expect("Failed to parse log message");
            if msg.ends_with('\n') {
                msg = &msg[..msg.len() - 1];
            }
            info!("{msg}");
        })
        .start()
        .await
        .map_err(|error| IoError(error.to_string()))?;
    let database_url = upload_test_file(&container).await?;

    let driver = rsql_driver_s3::Driver;
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
    let port = container
        .get_host_port_ipv4(4566)
        .await
        .map_err(|error| IoError(error.to_string()))?;
    let file_name = "users.csv";
    let endpoint_url = format!("http://{HOST}:{port}");
    let credentials = Credentials::from_keys(ACCESS_KEY_ID, SECRET_ACCESS_KEY, None);

    let config = aws_sdk_s3::config::Builder::default()
        .region(Region::new(REGION))
        .credentials_provider(credentials)
        .endpoint_url(&endpoint_url)
        .build();
    let client = Client::from_conf(config);

    client
        .create_bucket()
        .bucket(BUCKET)
        .send()
        .await
        .map_err(|error| IoError(error.to_string()))?;

    let crate_directory = env!("CARGO_MANIFEST_DIR");
    let file_path = PathBuf::from(crate_directory)
        .join("..")
        .join("..")
        .join("datasets")
        .join(file_name);
    let byte_stream = ByteStream::from_path(file_path)
        .await
        .map_err(|error| IoError(error.to_string()))?;

    client
        .put_object()
        .bucket(BUCKET)
        .key(file_name)
        .body(byte_stream)
        .send()
        .await
        .map_err(|error| IoError(error.to_string()))?;

    let database_url = format!(
        "s3://{ACCESS_KEY_ID}:{SECRET_ACCESS_KEY}@{HOST}:{port}/{BUCKET}/{file_name}?region={REGION}&scheme=http",
    );
    Ok(database_url)
}
