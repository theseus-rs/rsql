#[allow(unused_imports)]
use indoc::indoc;
use rsql_driver::{Driver, Result, Value};
#[allow(unused_imports)]
use rsql_driver_test_utils::dataset_url;
use rsql_drivers::DriverManager;

#[tokio::test(flavor = "multi_thread")]
async fn test_file_drivers() -> Result<()> {
    DriverManager::initialize()?;
    let database_urls: Vec<(String, Option<&str>)> = vec![
        #[cfg(feature = "driver-arrow")]
        (dataset_url("file", "users.arrow"), None),
        #[cfg(feature = "driver-avro")]
        (dataset_url("file", "users.avro"), None),
        #[cfg(feature = "driver-brotli")]
        (dataset_url("file", "users.csv.br"), None),
        #[cfg(feature = "driver-bzip2")]
        (dataset_url("file", "users.csv.bz2"), None),
        #[cfg(feature = "driver-csv")]
        (dataset_url("file", "users.csv"), None),
        #[cfg(feature = "driver-duckdb")]
        (dataset_url("file", "users.duckdb"), None),
        #[cfg(feature = "driver-excel")]
        (dataset_url("file", "users.xlsx"), None),
        #[cfg(feature = "driver-gzip")]
        (dataset_url("file", "users.csv.gz"), None),
        #[cfg(feature = "driver-json")]
        (dataset_url("file", "users.json"), None),
        #[cfg(feature = "driver-jsonl")]
        (dataset_url("file", "users.jsonl"), None),
        #[cfg(feature = "driver-lz4")]
        (dataset_url("file", "users.csv.lz4"), None),
        #[cfg(feature = "driver-ods")]
        (dataset_url("file", "users.ods"), None),
        #[cfg(feature = "driver-orc")]
        (dataset_url("file", "users.orc"), None),
        #[cfg(feature = "driver-parquet")]
        (dataset_url("file", "users.parquet"), None),
        #[cfg(feature = "driver-sqlite")]
        (dataset_url("file", "users.sqlite3"), None),
        #[cfg(feature = "driver-tsv")]
        (dataset_url("file", "users.tsv"), None),
        #[cfg(feature = "driver-xml")]
        (
            dataset_url("file", "users.xml"),
            Some(indoc! {r#"
                WITH cte_user AS (
                    SELECT unnest("data"."user") AS "user" FROM users
                )
                SELECT "user".* FROM cte_user
            "#}),
        ),
        #[cfg(feature = "driver-xz")]
        (dataset_url("file", "users.csv.xz"), None),
        #[cfg(feature = "driver-yaml")]
        (dataset_url("file", "users.yaml"), None),
        #[cfg(feature = "driver-zstd")]
        (dataset_url("file", "users.csv.zst"), None),
    ];
    for (database_url, sql) in database_urls {
        test_file_driver(database_url.as_str(), sql).await?;
    }
    Ok(())
}

async fn test_file_driver(database_url: &str, sql: Option<&str>) -> Result<()> {
    let sql = sql.unwrap_or("SELECT id, name FROM users ORDER BY id");
    let driver = rsql_driver_file::Driver;
    let mut connection = driver.connect(database_url).await?;

    let mut query_result = connection.query(sql, &[]).await?;

    assert_eq!(query_result.columns(), vec!["id", "name"]);
    assert_eq!(
        query_result.next().await.cloned(),
        Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
    );
    assert_eq!(
        query_result.next().await.cloned(),
        Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
    );
    assert!(query_result.next().await.is_none());

    connection.close().await?;
    Ok(())
}
