use anyhow::{Context, Result, ensure};
use rsql_driver::{Driver, StatementMetadata, Value};

#[tokio::test(flavor = "multi_thread")]
async fn in_memory_database() -> Result<()> {
    let mut connection = rsql_driver_h2::Driver.connect("h2://").await?;
    ensure!(connection.url() == "h2://", "connection URL changed");
    ensure!(
        connection
            .execute(
                "CREATE TABLE person (id INTEGER PRIMARY KEY, name VARCHAR(100))",
                &[]
            )
            .await?
            == 0,
        "creating a table should affect no rows"
    );
    ensure!(
        connection
            .execute(
                "INSERT INTO person VALUES (?, ?)",
                &[&1_i32, &"O'Reilly 🦀"]
            )
            .await?
            == 1,
        "inserting a person should affect one row"
    );
    let mut result = connection
        .query("SELECT id, name FROM person WHERE id = ?", &[&1_i32])
        .await?;
    ensure!(
        result.columns() == ["ID", "NAME"],
        "unexpected person columns"
    );
    ensure!(
        result.next().await == Some(&vec![Value::I32(1), Value::String("O'Reilly 🦀".into())]),
        "unexpected person row"
    );
    ensure!(result.next().await.is_none(), "unexpected extra person row");
    let mut version = connection.query("SELECT H2VERSION()", &[]).await?;
    ensure!(
        version.next().await == Some(&vec![Value::String(rsql_driver_h2::H2_VERSION.into())]),
        "unexpected H2 version"
    );
    let metadata = connection.metadata().await?;
    let schema = metadata
        .current_catalog_schema()
        .context("Missing current schema")?;
    ensure!(
        schema.get("PERSON").is_some(),
        "missing PERSON table metadata"
    );
    ensure!(
        connection
            .query("SELECT missing FROM person", &[])
            .await
            .is_err(),
        "querying a missing column should fail"
    );
    connection.close().await?;
    connection.close().await?;
    ensure!(
        connection.query("SELECT 1", &[]).await.is_err(),
        "querying a closed connection should fail"
    );
    let mut fresh = rsql_driver_h2::Driver.connect("h2://").await?;
    ensure!(
        fresh.query("SELECT * FROM person", &[]).await.is_err(),
        "a fresh in-memory database should not contain the person table"
    );
    fresh.close().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn persistent_database() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let database = directory.path().join("database");
    let url = format!("h2:{};MODE=PostgreSQL", database.to_string_lossy());
    let mut connection = rsql_driver_h2::Driver.connect(&url).await?;
    connection
        .execute("CREATE TABLE persisted(id INTEGER)", &[])
        .await?;
    connection
        .execute("INSERT INTO persisted VALUES (?)", &[&42_i32])
        .await?;
    connection.close().await?;
    let mut connection = rsql_driver_h2::Driver.connect(&url).await?;
    let mut result = connection.query("SELECT id FROM persisted", &[]).await?;
    ensure!(
        result.next().await == Some(&vec![Value::I32(42)]),
        "the row should persist after reconnecting"
    );
    connection.close().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn json_documents() -> Result<()> {
    let mut connection = rsql_driver_h2::Driver.connect("h2://").await?;
    connection
        .execute("CREATE TABLE documents (doc JSON)", &[])
        .await?;
    for document in [
        Value::Map(std::iter::empty().collect()),
        Value::Map(
            [
                (
                    Value::String("name".into()),
                    Value::String("O'Reilly 🦀".into()),
                ),
                (
                    Value::String("nested".into()),
                    Value::Map(
                        [(
                            Value::String("values".into()),
                            Value::Array(vec![Value::Bool(true), Value::Null]),
                        )]
                        .into_iter()
                        .collect(),
                    ),
                ),
            ]
            .into_iter()
            .collect(),
        ),
    ] {
        connection.execute("DELETE FROM documents", &[]).await?;
        connection
            .execute("INSERT INTO documents VALUES (?)", &[&document])
            .await?;
        let mut result = connection
            .query("SELECT doc, doc IS JSON OBJECT FROM documents", &[])
            .await?;
        let row = result.next().await.context("missing JSON document")?;
        ensure!(
            row == &vec![document, Value::Bool(true)],
            "unexpected JSON document: {row:?}"
        );
    }
    connection.close().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn time_arrays() -> Result<()> {
    // H2-specific bindings also apply when using the generic JDBC driver.
    let url = format!(
        "jdbc:h2:mem:?driver=org.h2.Driver&dependency=com.h2database:h2:{}",
        rsql_driver_h2::H2_VERSION
    );
    let mut connection = rsql_driver_jdbc::Driver.connect(&url).await?;
    let time = Value::Time("12:34:56.123456789".parse()?);
    for values in [vec![time.clone(), Value::Null], vec![], vec![Value::Null]] {
        let values = Value::Array(values);
        let mut result = connection
            .query("SELECT CAST(? AS TIME(9) ARRAY)", &[&values])
            .await?;
        let row = result.next().await.context("missing time array")?;
        ensure!(row == &vec![values], "unexpected time array: {row:?}");
    }
    let values = Value::Array(vec![Value::Array(vec![time, Value::Null])]);
    let mut result = connection
        .query("SELECT CAST(? AS TIME(9) ARRAY ARRAY)", &[&values])
        .await?;
    let row = result.next().await.context("missing nested time array")?;
    ensure!(
        row == &vec![values],
        "unexpected nested time array: {row:?}"
    );
    connection.close().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn show_and_call_queries() -> Result<()> {
    let mut connection = rsql_driver_h2::Driver.connect("h2://").await?;
    connection
        .execute("CREATE TABLE person (id INTEGER)", &[])
        .await?;
    for sql in [
        "SHOW TABLES",
        "show schemas",
        "/* columns */ SHOW COLUMNS FROM person",
        "-- version\nCALL H2VERSION()",
    ] {
        ensure!(
            matches!(connection.parse_sql(sql), StatementMetadata::Query),
            "not classified as a query: {sql}"
        );
        let mut result = connection.query(sql, &[]).await?;
        let row = result.next().await.context("missing SHOW/CALL result")?;
        if sql.contains("H2VERSION") {
            ensure!(
                row == &vec![Value::String(rsql_driver_h2::H2_VERSION.into())],
                "unexpected H2 version: {row:?}"
            );
        }
    }
    ensure!(matches!(
        connection.parse_sql("INSERT INTO person VALUES (1)"),
        StatementMetadata::DML
    ));
    connection.close().await?;
    Ok(())
}
