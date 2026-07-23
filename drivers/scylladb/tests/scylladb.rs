#![cfg_attr(
    target_os = "linux",
    expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic when verification fails"
    )
)]

#[cfg(target_os = "linux")]
use rsql_driver::{DriverManager, Value};
#[cfg(target_os = "linux")]
use std::sync::Arc;
#[cfg(target_os = "linux")]
use testcontainers::{ImageExt, core::IntoContainerPort, runners::AsyncRunner};

#[cfg(target_os = "linux")]
const SCYLLADB_PORT: u16 = 9042;

#[cfg(target_os = "linux")]
// Scylla advertises its CQL address during topology discovery. Keep the advertised address
// aligned with the host mapping so the driver does not switch to a container-only endpoint.
#[tokio::test]
async fn scylladb_driver_round_trip_and_metadata() -> anyhow::Result<()> {
    let container = testcontainers_modules::scylladb::ScyllaDB::default()
        .with_cmd([
            "--smp",
            "1",
            "--memory",
            "1G",
            "--overprovisioned",
            "1",
            "--broadcast-rpc-address",
            "127.0.0.1",
        ])
        .with_mapped_port(SCYLLADB_PORT, SCYLLADB_PORT.tcp())
        .start()
        .await?;
    let url = format!("scylladb://127.0.0.1:{SCYLLADB_PORT}");

    DriverManager::add(Arc::new(rsql_driver_scylladb::Driver))?;
    let mut connection = DriverManager::connect(&url).await?;
    assert_eq!(connection.url(), &url);

    connection
        .execute(
            "CREATE KEYSPACE rsql_test WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}",
            &[],
        )
        .await?;
    connection.execute("USE rsql_test", &[]).await?;
    connection
        .execute(
            "CREATE TABLE users (tenant int, id int, name text, active boolean, tags list<text>, PRIMARY KEY (tenant, id))",
            &[],
        )
        .await?;
    connection
        .execute("CREATE INDEX users_name_idx ON users (name)", &[])
        .await?;
    connection
        .execute(
            "CREATE MATERIALIZED VIEW users_by_name AS SELECT * FROM users WHERE name IS NOT NULL AND tenant IS NOT NULL AND id IS NOT NULL PRIMARY KEY (name, tenant, id)",
            &[],
        )
        .await?;

    let tags = Value::Array(vec![
        Value::String("admin".to_string()),
        Value::String("active".to_string()),
    ]);
    let affected = connection
        .execute(
            "INSERT INTO users (tenant, id, name, active, tags) VALUES (?, ?, ?, ?, ?)",
            &[&1_i32, &7_i32, &"Alice", &true, &tags],
        )
        .await?;
    assert_eq!(affected, 0);
    let mut result = connection
        .query(
            "SELECT name, active, tags FROM users WHERE tenant = ? AND id = ?",
            &[&1_i32, &7_i32],
        )
        .await?;
    assert_eq!(result.columns(), ["name", "active", "tags"]);
    assert_eq!(
        result.next().await,
        Some(&vec![
            Value::String("Alice".to_string()),
            Value::Bool(true),
            tags,
        ])
    );

    let metadata = connection.metadata().await?;
    let catalog = metadata
        .current_catalog()
        .ok_or_else(|| anyhow::anyhow!("missing ScyllaDB catalog"))?;
    let schema = catalog
        .current_schema()
        .ok_or_else(|| anyhow::anyhow!("USE did not select rsql_test"))?;
    assert_eq!(schema.name(), "rsql_test");
    assert!(schema.get_view("users_by_name").is_some());

    let users = schema
        .get("users")
        .ok_or_else(|| anyhow::anyhow!("missing users table"))?;
    assert_eq!(
        users
            .primary_key()
            .ok_or_else(|| anyhow::anyhow!("missing primary key"))?
            .columns(),
        ["tenant", "id"]
    );
    let indexes = users
        .indexes()
        .iter()
        .map(|index| index.name())
        .collect::<Vec<_>>();
    assert!(indexes.contains(&"PRIMARY"));
    assert!(indexes.contains(&"users_name_idx"));
    assert_eq!(
        users.get_column("tags").map(rsql_driver::Column::data_type),
        Some("list<text>")
    );

    connection.close().await?;
    container.stop().await?;
    container.rm().await?;
    Ok(())
}
