#[cfg(target_os = "linux")]
use rsql_driver::Driver;
#[cfg(target_os = "linux")]
use testcontainers::runners::AsyncRunner;

#[cfg(target_os = "linux")]
#[tokio::test]
async fn test_mariadb_driver() -> anyhow::Result<()> {
    let image =
        testcontainers::ContainerRequest::from(testcontainers_modules::mariadb::Mariadb::default());
    let container = image.start().await?;
    let port = container.get_host_port_ipv4(3306).await?;

    let database_url = format!("mariadb://root@127.0.0.1:{port}/test");
    let mut connection = rsql_driver_mariadb::Driver
        .connect(database_url.as_str())
        .await?;
    assert_eq!(database_url, connection.url().as_str());

    test_schema(&mut *connection).await?;

    container.stop().await?;
    container.rm().await?;
    Ok(())
}

#[cfg(target_os = "linux")]
async fn test_schema(connection: &mut dyn rsql_driver::Connection) -> anyhow::Result<()> {
    let _ = connection
        .execute("CREATE TABLE contacts (id INT PRIMARY KEY, email VARCHAR(20))")
        .await?;
    let _ = connection
        .execute("CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(20))")
        .await?;

    let metadata = connection.metadata().await?;
    let schema = metadata.current_schema().expect("schema");
    let tables = schema
        .tables()
        .iter()
        .map(|table| table.name())
        .collect::<Vec<_>>();
    assert!(tables.contains(&"contacts"));
    assert!(tables.contains(&"users"));

    let contacts_table = schema.get("contacts").expect("contacts table");
    let contacts_indexes = contacts_table
        .indexes()
        .iter()
        .map(|index| index.name())
        .collect::<Vec<_>>();
    assert_eq!(contacts_indexes, vec!["PRIMARY"]);

    let user_table = schema.get("users").expect("users table");
    let user_indexes = user_table
        .indexes()
        .iter()
        .map(|index| index.name())
        .collect::<Vec<_>>();
    assert_eq!(user_indexes, vec!["PRIMARY"]);

    Ok(())
}
