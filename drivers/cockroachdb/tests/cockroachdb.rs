#[cfg(target_os = "linux")]
use rsql_driver::{Driver, Value};
#[cfg(target_os = "linux")]
use testcontainers::runners::AsyncRunner;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_cockroachdb_driver() -> anyhow::Result<()> {
    let image = testcontainers::ContainerRequest::from(
        testcontainers_modules::cockroach_db::CockroachDb::default(),
    );
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(26257).await?;

    let database_url = format!("cockroachdb://root:postgres@localhost:{port}/defaultdb");
    let mut connection = rsql_driver_cockroachdb::Driver
        .connect(database_url.as_str())
        .await?;
    assert_eq!(database_url, connection.url().as_str());

    let mut query_result = connection.query("SELECT 1", &[]).await?;
    let row = query_result.next().await.expect("no row");
    let value = row.first().expect("no value");

    assert_eq!(*value, Value::I64(1));

    container.stop().await?;
    container.rm().await?;
    Ok(())
}
