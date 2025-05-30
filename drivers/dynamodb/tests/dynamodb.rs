use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::config::Region;
use aws_sdk_dynamodb::types::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
    ScalarAttributeType,
};
use rsql_driver::Error::IoError;
use rsql_driver::{Driver, Result, Value};
use std::env;
use testcontainers_modules::dynamodb_local::DynamoDb;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::ImageExt;
use testcontainers_modules::testcontainers::core::logs::LogFrame;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use tracing::info;
use tracing_subscriber::EnvFilter;

static ACCESS_KEY_ID: &str = "test";
static SECRET_ACCESS_KEY: &str = "test";
static REGION: &str = "us-east-1";

#[tokio::test]
async fn test_dynamodb_driver() -> Result<()> {
    if env::var("CI").unwrap_or_default() == "true" && env::consts::OS != "linux" {
        eprintln!("Skipping CI test on non-linux platform");
        return Ok(());
    }

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::new("trace")
                .add_directive("aws_config=trace".parse().expect("Invalid directive"))
                .add_directive("aws_sdk_dynamodb=trace".parse().expect("Invalid directive")),
        )
        .with_test_writer()
        .compact()
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let container = DynamoDb::default()
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
    let database_url = setup_table(&container).await?;

    let driver = rsql_driver_dynamodb::Driver;
    let mut connection = driver.connect(database_url.as_str()).await?;

    let mut query_result = connection
        .query("SELECT id, name FROM users WHERE id = 1")
        .await?;
    let columns = query_result.columns().await;
    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"name".to_string()));

    let row = query_result.next().await.expect("No row found").to_vec();
    assert!(row.contains(&Value::I128(1)));
    assert!(row.contains(&Value::String("John Doe".to_string())));
    assert!(query_result.next().await.is_none());

    test_schema(&mut *connection).await?;

    connection.close().await?;
    Ok(())
}

async fn test_schema(connection: &mut dyn rsql_driver::Connection) -> Result<()> {
    let metadata = connection.metadata().await?;
    let catalog = metadata.current_catalog().expect("catalog");
    let schema = catalog.current_schema().expect("schema");
    let tables = schema
        .tables()
        .iter()
        .map(|table| table.name())
        .collect::<Vec<_>>();
    assert!(tables.contains(&"users"));

    let user_table = schema.get("users").expect("users table");
    let user_indexes = user_table
        .indexes()
        .iter()
        .map(|index| index.name())
        .collect::<Vec<_>>();
    assert!(user_indexes.contains(&"id"));
    assert!(user_indexes.contains(&"name"));

    Ok(())
}

async fn setup_table(container: &ContainerAsync<DynamoDb>) -> Result<String> {
    let host = container
        .get_host()
        .await
        .map_err(|error| IoError(error.to_string()))?;
    let port = container
        .get_host_port_ipv4(8000)
        .await
        .map_err(|error| IoError(error.to_string()))?;
    let table_name = "users";
    let endpoint_url = format!("http://{host}:{port}");
    let credentials = Credentials::from_keys(ACCESS_KEY_ID, SECRET_ACCESS_KEY, None);

    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(REGION))
        .endpoint_url(endpoint_url)
        .credentials_provider(credentials)
        .load()
        .await;
    let client = Client::new(&shared_config);

    let id_key = KeySchemaElement::builder()
        .attribute_name("id".to_string())
        .key_type(KeyType::Hash)
        .build()
        .map_err(|error| IoError(format!("{error:?}")))?;
    let id_attribute = AttributeDefinition::builder()
        .attribute_name("id".to_string())
        .attribute_type(ScalarAttributeType::N)
        .build()
        .map_err(|error| IoError(format!("{error:?}")))?;

    let name_key = KeySchemaElement::builder()
        .attribute_name("name".to_string())
        .key_type(KeyType::Range)
        .build()
        .map_err(|error| IoError(format!("{error:?}")))?;
    let name_attribute = AttributeDefinition::builder()
        .attribute_name("name".to_string())
        .attribute_type(ScalarAttributeType::S)
        .build()
        .map_err(|error| IoError(format!("{error:?}")))?;

    let provisioned_throughput = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build()
        .map_err(|error| IoError(format!("{error:?}")))?;

    let _table = client
        .create_table()
        .table_name(table_name)
        .key_schema(id_key)
        .attribute_definitions(id_attribute)
        .key_schema(name_key)
        .attribute_definitions(name_attribute)
        .provisioned_throughput(provisioned_throughput)
        .send()
        .await
        .map_err(|error| IoError(format!("{error:?}")))?;

    let _ = client
        .put_item()
        .table_name(table_name)
        .item("id", AttributeValue::N("1".to_string()))
        .item("name", AttributeValue::S("John Doe".to_string()))
        .send()
        .await
        .map_err(|error| IoError(format!("{error:?}")))?;
    let _ = client
        .put_item()
        .table_name(table_name)
        .item("id", AttributeValue::N("2".to_string()))
        .item("name", AttributeValue::S("Jane Smith".to_string()))
        .send()
        .await
        .map_err(|error| IoError(format!("{error:?}")))?;

    let database_url = format!(
        "dynamodb://{ACCESS_KEY_ID}:{SECRET_ACCESS_KEY}@{host}:{port}?region={REGION}&scheme=http"
    );
    Ok(database_url)
}
