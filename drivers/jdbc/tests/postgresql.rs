#![cfg(not(all(target_os = "windows", target_arch = "aarch64")))]

use anyhow::{Context, Result, ensure};
use postgresql_embedded::{PostgreSQL, Settings};
use rsql_driver::{Connection, Driver, StatementMetadata, Value};
use std::time::Duration;

const JDBC_VERSION: &str = "42.7.13";

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn postgresql_jdbc() -> Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let settings = Settings {
        version: "=18.6.0".parse()?,
        timeout: Some(Duration::from_secs(60)),
        ..Settings::default()
    };
    let mut postgresql = PostgreSQL::new(settings);
    postgresql.setup().await?;
    postgresql.start().await?;
    let outcome = async {
        postgresql.create_database("jdbc_test").await?;
        let settings = postgresql.settings();
        let parameters = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("user", &settings.username)
            .append_pair("password", &settings.password)
            .append_pair(
                "dependency",
                &format!("org.postgresql:postgresql:{JDBC_VERSION}"),
            )
            .append_pair("driver", "org.postgresql.Driver")
            .append_pair("sslmode", "disable")
            .append_pair("connectTimeout", "5")
            .finish();
        let url = format!(
            "jdbc:postgresql://{}:{}/jdbc_test?{parameters}",
            settings.host, settings.port
        );
        exercise_driver(&url).await
    }
    .await;
    postgresql.stop().await?;
    outcome
}

async fn exercise_driver(url: &str) -> Result<()> {
    let driver = rsql_driver_jdbc::Driver;
    assert_eq!(driver.identifier(), "jdbc");
    let mut connection = driver.connect(url).await?;
    assert_eq!(connection.url(), url);
    let outcome = async {
        connection.execute("SET TIME ZONE 'UTC'", &[]).await?;
        query_and_execute(connection.as_mut())
            .await
            .context("query/execute")?;
        scalar_types(connection.as_mut())
            .await
            .context("scalar results")?;
        parameter_types(connection.as_mut())
            .await
            .context("parameters")?;
        array_types(connection.as_mut()).await.context("arrays")?;
        transactions(connection.as_mut())
            .await
            .context("transactions")?;
        metadata(connection.as_mut()).await.context("metadata")?;
        errors(connection.as_mut()).await.context("errors")?;
        Result::<()>::Ok(())
    }
    .await;
    connection.close().await?;
    outcome?;
    connection.close().await?;
    assert!(connection.query("SELECT 1", &[]).await.is_err());
    assert!(connection.execute("SELECT 1", &[]).await.is_err());
    assert!(connection.metadata().await.is_err());

    let discovered_url = url.replace("&driver=org.postgresql.Driver", "");
    let mut discovered = driver
        .connect(&discovered_url)
        .await
        .context("DriverManager discovery")?;
    assert_eq!(
        scalar(discovered.as_mut(), "SELECT 42", None).await?,
        Value::I32(42)
    );
    discovered.close().await?;
    assert!(
        driver
            .connect(&url.replace("org.postgresql.Driver", "missing.Driver"))
            .await
            .is_err()
    );
    assert!(
        driver
            .connect(&url.replacen("jdbc:postgresql:", "jdbc:unsupported:", 1))
            .await
            .is_err()
    );
    Ok(())
}

async fn scalar(
    connection: &mut dyn Connection,
    sql: &str,
    parameter: Option<&Value>,
) -> Result<Value> {
    let parameters: Vec<&dyn rsql_driver::ToSql> = parameter
        .into_iter()
        .map(|value| value as &dyn rsql_driver::ToSql)
        .collect();
    let mut result = connection
        .query(sql, &parameters)
        .await
        .with_context(|| sql.to_string())?;
    assert_eq!(result.columns().len(), 1, "{sql}");
    let row = result.next().await.context("Missing row")?;
    assert_eq!(row.len(), 1, "{sql}");
    let value = row.first().context("Missing value")?.clone();
    assert!(result.next().await.is_none(), "{sql}");
    Ok(value)
}

async fn query_and_execute(connection: &mut dyn Connection) -> Result<()> {
    assert!(matches!(
        connection.parse_sql("CREATE TABLE sample(id INT)"),
        StatementMetadata::DDL
    ));
    assert!(matches!(
        connection.parse_sql("SELECT 1"),
        StatementMetadata::Query
    ));
    assert!(matches!(
        connection.parse_sql("INSERT INTO sample VALUES (1)"),
        StatementMetadata::DML
    ));
    assert_eq!(
        connection
            .execute(
                "CREATE TABLE sample(id INTEGER PRIMARY KEY, name TEXT)",
                &[]
            )
            .await?,
        0
    );
    assert_eq!(
        connection
            .execute(
                "INSERT INTO sample VALUES (?, ?)",
                &[&1_i32, &"O'Reilly 🦀 ?"]
            )
            .await?,
        1
    );
    assert_eq!(
        connection
            .execute("INSERT INTO sample VALUES (2, 'two'), (3, 'three')", &[])
            .await?,
        2
    );
    let mut rows = connection
        .query(
            "SELECT id AS \"Identifier\", name FROM sample ORDER BY id",
            &[],
        )
        .await?;
    assert_eq!(rows.columns(), ["Identifier", "name"]);
    assert_eq!(
        rows.next().await,
        Some(&vec![Value::I32(1), Value::String("O'Reilly 🦀 ?".into())])
    );
    assert_eq!(
        rows.next().await,
        Some(&vec![Value::I32(2), Value::String("two".into())])
    );
    assert_eq!(
        rows.next().await,
        Some(&vec![Value::I32(3), Value::String("three".into())])
    );
    assert!(rows.next().await.is_none());
    assert!(rows.next().await.is_none());
    let mut empty = connection
        .query("SELECT id, name FROM sample WHERE false", &[])
        .await?;
    assert_eq!(empty.columns(), ["id", "name"]);
    assert!(empty.next().await.is_none());
    assert_eq!(
        connection
            .execute(
                "UPDATE sample SET name = ? WHERE id = ?",
                &[&"updated", &2_i32]
            )
            .await?,
        1
    );
    assert_eq!(
        connection
            .execute("DELETE FROM sample WHERE id > ?", &[&1_i32])
            .await?,
        2
    );
    assert_eq!(
        connection
            .execute("DELETE FROM sample WHERE id = 999", &[])
            .await?,
        0
    );
    let mut returning = connection
        .query("INSERT INTO sample VALUES (4, 'four') RETURNING id", &[])
        .await?;
    assert_eq!(returning.next().await, Some(&vec![Value::I32(4)]));
    assert!(returning.next().await.is_none());
    result_shapes(connection).await
}

async fn result_shapes(connection: &mut dyn Connection) -> Result<()> {
    let mut duplicate_labels = connection
        .query("SELECT 1 AS same, 2 AS same, NULL AS empty", &[])
        .await?;
    assert_eq!(duplicate_labels.columns(), ["same", "same", "empty"]);
    assert_eq!(
        duplicate_labels.next().await,
        Some(&vec![Value::I32(1), Value::I32(2), Value::Null])
    );
    let mut mixed_nulls = connection.query("SELECT n, CASE WHEN n = 2 THEN NULL ELSE n END AS nullable FROM generate_series(1, 3) AS n", &[]).await?;
    for (n, value) in [(1, Value::I32(1)), (2, Value::Null), (3, Value::I32(3))] {
        assert_eq!(mixed_nulls.next().await, Some(&vec![Value::I32(n), value]));
    }
    assert!(mixed_nulls.next().await.is_none());
    Ok(())
}

async fn scalar_types(connection: &mut dyn Connection) -> Result<()> {
    let cases = [
        ("true::boolean", Value::Bool(true)),
        ("false::boolean", Value::Bool(false)),
        ("'-32768'::smallint", Value::I16(i16::MIN)),
        ("'32767'::smallint", Value::I16(i16::MAX)),
        ("'-2147483648'::integer", Value::I32(i32::MIN)),
        ("'2147483647'::integer", Value::I32(i32::MAX)),
        ("'-9223372036854775808'::bigint", Value::I64(i64::MIN)),
        ("'9223372036854775807'::bigint", Value::I64(i64::MAX)),
        ("1.25::real", Value::F32(1.25)),
        ("-1.25::double precision", Value::F64(-1.25)),
        ("123.456::numeric", Value::Decimal("123.456".parse()?)),
        (
            "1234567890123456789012345678901234567890::numeric",
            Value::String("1234567890123456789012345678901234567890".into()),
        ),
        ("'hello'::char(5)", Value::String("hello".into())),
        ("'héllo 🦀'::varchar", Value::String("héllo 🦀".into())),
        ("''::text", Value::String(String::new())),
        (
            "decode('00017f80ff', 'hex')",
            Value::Bytes(vec![0, 1, 127, 128, 255]),
        ),
        ("''::bytea", Value::Bytes(vec![])),
        ("'2024-02-29'::date", Value::Date("2024-02-29".parse()?)),
        (
            "'12:34:56.123456'::time",
            Value::Time("12:34:56.123456".parse()?),
        ),
        (
            "'2024-02-29 12:34:56.123456'::timestamp",
            Value::DateTime("2024-02-29T12:34:56.123456".parse()?),
        ),
        (
            "'123e4567-e89b-12d3-a456-426614174000'::uuid",
            Value::Uuid("123e4567-e89b-12d3-a456-426614174000".parse()?),
        ),
        (
            "'{\"a\":[1,true,null],\"b\":\"text\"}'::jsonb",
            Value::from(serde_json::json!({"a": [1, true, null], "b": "text"})),
        ),
        (
            "'[1,2,null]'::json",
            Value::from(serde_json::json!([1, 2, null])),
        ),
        ("B'10101'::bit(5)", Value::String("10101".into())),
        ("B'101'::varbit", Value::String("101".into())),
        ("'<root/>'::xml", Value::String("<root/>".into())),
    ];
    for (expression, expected) in cases {
        let sql = format!("SELECT {expression}");
        assert_eq!(scalar(connection, &sql, None).await?, expected, "{sql}");
    }
    postgres_types(connection).await?;
    null_types(connection).await?;
    custom_types(connection).await
}

async fn postgres_types(connection: &mut dyn Connection) -> Result<()> {
    let cases = [
        (
            "'12:34:56.123456+05:30'::timetz",
            Value::String("12:34:56.123456+05:30".into()),
        ),
        (
            "'2024-02-29 12:34:56.123456+00'::timestamptz",
            Value::String("2024-02-29 12:34:56.123456+00".into()),
        ),
        (
            "'1 year 2 mons 3 days 04:05:06'::interval",
            Value::String("1 year 2 mons 3 days 04:05:06".into()),
        ),
        ("'192.0.2.1'::inet", Value::String("192.0.2.1".into())),
        ("'192.0.2.0/24'::cidr", Value::String("192.0.2.0/24".into())),
        (
            "'08:00:2b:01:02:03'::macaddr",
            Value::String("08:00:2b:01:02:03".into()),
        ),
        ("'(1,2)'::point", Value::String("(1,2)".into())),
        ("'[1,5)'::int4range", Value::String("[1,5)".into())),
        (
            "'{[1,5),[8,10)}'::int4multirange",
            Value::String("{[1,5),[8,10)}".into()),
        ),
        ("42::oid", Value::I64(42)),
        ("'NaN'::numeric", Value::String("NaN".into())),
    ];
    for (expression, expected) in cases {
        let sql = format!("SELECT {expression}");
        assert_eq!(scalar(connection, &sql, None).await?, expected, "{sql}");
    }
    Ok(())
}

async fn null_types(connection: &mut dyn Connection) -> Result<()> {
    for datatype in [
        "boolean",
        "smallint",
        "integer",
        "bigint",
        "real",
        "double precision",
        "numeric",
        "text",
        "bytea",
        "date",
        "time",
        "timestamp",
        "uuid",
        "json",
        "jsonb",
        "integer[]",
        "timetz",
        "timestamptz",
        "interval",
        "bit",
        "varbit",
        "xml",
        "inet",
        "cidr",
        "macaddr",
        "point",
        "int4range",
        "int4multirange",
        "oid",
    ] {
        assert_eq!(
            scalar(connection, &format!("SELECT NULL::{datatype}"), None).await?,
            Value::Null,
            "NULL::{datatype}"
        );
        assert_eq!(
            scalar(
                connection,
                &format!("SELECT CAST(? AS {datatype})"),
                Some(&Value::Null)
            )
            .await?,
            Value::Null,
            "bound NULL::{datatype}"
        );
    }
    Ok(())
}

async fn custom_types(connection: &mut dyn Connection) -> Result<()> {
    assert_eq!(
        scalar(connection, "SELECT 'Infinity'::double precision", None).await?,
        Value::F64(f64::INFINITY)
    );
    assert!(
        matches!(scalar(connection, "SELECT 'NaN'::real", None).await?, Value::F32(value) if value.is_nan())
    );
    connection
        .execute("CREATE TYPE mood AS ENUM ('happy', 'sad')", &[])
        .await?;
    connection
        .execute(
            "CREATE DOMAIN positive_int AS integer CHECK (VALUE > 0)",
            &[],
        )
        .await?;
    connection
        .execute("CREATE TYPE pair AS (n integer, label text)", &[])
        .await?;
    for (expression, expected) in [
        ("'happy'::mood", Value::String("happy".into())),
        ("42::positive_int", Value::I32(42)),
        ("ROW(1, 'one')::pair", Value::String("(1,one)".into())),
    ] {
        assert_eq!(
            scalar(connection, &format!("SELECT {expression}"), None).await?,
            expected,
            "{expression}"
        );
    }
    Ok(())
}

async fn parameter_types(connection: &mut dyn Connection) -> Result<()> {
    let cases = [
        ("boolean", Value::Bool(true), Value::Bool(true)),
        (
            "smallint",
            Value::I8(i8::MIN),
            Value::I16(i16::from(i8::MIN)),
        ),
        ("smallint", Value::I16(i16::MIN), Value::I16(i16::MIN)),
        ("integer", Value::I32(i32::MIN), Value::I32(i32::MIN)),
        ("bigint", Value::I64(i64::MIN), Value::I64(i64::MIN)),
        (
            "numeric",
            Value::I128(i128::MAX),
            Value::String(i128::MAX.to_string()),
        ),
        (
            "smallint",
            Value::U8(u8::MAX),
            Value::I16(i16::from(u8::MAX)),
        ),
        (
            "integer",
            Value::U16(u16::MAX),
            Value::I32(i32::from(u16::MAX)),
        ),
        (
            "bigint",
            Value::U32(u32::MAX),
            Value::I64(i64::from(u32::MAX)),
        ),
        (
            "numeric",
            Value::U64(u64::MAX),
            Value::Decimal(u64::MAX.into()),
        ),
        (
            "numeric",
            Value::U128(u128::MAX),
            Value::String(u128::MAX.to_string()),
        ),
        ("real", Value::F32(1.25), Value::F32(1.25)),
        ("double precision", Value::F64(-1.25), Value::F64(-1.25)),
    ];
    for (datatype, input, expected) in cases {
        assert_eq!(
            scalar(
                connection,
                &format!("SELECT CAST(? AS {datatype})"),
                Some(&input)
            )
            .await?,
            expected,
            "{datatype}: {input:?}"
        );
    }
    let roundtrips = [
        (
            "text",
            Value::String("'); DROP TABLE sample; -- 🦀 ?".into()),
        ),
        ("bytea", Value::Bytes(vec![0, 127, 128, 255])),
        ("numeric", Value::Decimal("123456.789012".parse()?)),
        ("date", Value::Date("2024-02-29".parse()?)),
        ("time", Value::Time("12:34:56.123456".parse()?)),
        (
            "timestamp",
            Value::DateTime("2024-02-29T12:34:56.123456".parse()?),
        ),
        (
            "uuid",
            Value::Uuid("123e4567-e89b-12d3-a456-426614174000".parse()?),
        ),
        (
            "jsonb",
            Value::from(serde_json::json!({"integer": 42, "null": null, "list": [true, "🦀"]})),
        ),
        ("integer", Value::Null),
    ];
    for (datatype, value) in roundtrips {
        let actual = scalar(
            connection,
            &format!("SELECT CAST(? AS {datatype})"),
            Some(&value),
        )
        .await?;
        if matches!(value, Value::Map(_)) {
            // PostgreSQL jsonb orders object keys; JSON object equality is order-independent.
            assert!(matches!(actual, Value::Map(_)));
            assert_eq!(serde_json::to_value(actual)?, serde_json::to_value(value)?);
        } else {
            assert_eq!(actual, value, "{datatype}");
        }
    }
    Ok(())
}

async fn array_types(connection: &mut dyn Connection) -> Result<()> {
    let cases = [
        ("smallint", vec![Value::I16(-1), Value::Null, Value::I16(2)]),
        ("integer", vec![Value::I32(-1), Value::Null, Value::I32(2)]),
        ("bigint", vec![Value::I64(i64::MIN), Value::I64(i64::MAX)]),
        (
            "boolean",
            vec![Value::Bool(false), Value::Bool(true), Value::Null],
        ),
        ("real", vec![Value::F32(1.25), Value::Null]),
        ("double precision", vec![Value::F64(1.25), Value::Null]),
        (
            "numeric",
            vec![Value::Decimal("12.34".parse()?), Value::Null],
        ),
        (
            "text",
            vec![
                Value::String("a,b\"\\ 🦀".into()),
                Value::String(String::new()),
                Value::Null,
            ],
        ),
        ("bytea", vec![Value::Bytes(vec![0, 128, 255]), Value::Null]),
        (
            "uuid",
            vec![
                Value::Uuid("123e4567-e89b-12d3-a456-426614174000".parse()?),
                Value::Null,
            ],
        ),
        (
            "date",
            vec![Value::Date("2024-02-29".parse()?), Value::Null],
        ),
        ("time", vec![Value::Time("12:34:56".parse()?), Value::Null]),
        (
            "timestamp",
            vec![Value::DateTime("2024-02-29T12:34:56".parse()?), Value::Null],
        ),
        ("integer", vec![]),
        ("integer", vec![Value::Null, Value::Null]),
    ];
    for (datatype, values) in cases {
        let value = Value::Array(values);
        assert_eq!(
            scalar(
                connection,
                &format!("SELECT CAST(? AS {datatype}[])"),
                Some(&value)
            )
            .await?,
            value,
            "{datatype}[]"
        );
    }
    assert_eq!(
        scalar(connection, "SELECT ARRAY[1, NULL, 3]", None).await?,
        Value::Array(vec![Value::I32(1), Value::Null, Value::I32(3)])
    );
    let nested = Value::Array(vec![
        Value::Array(vec![Value::I32(1), Value::Null]),
        Value::Array(vec![Value::I32(2), Value::I32(3)]),
    ]);
    assert_eq!(
        scalar(connection, "SELECT CAST(? AS integer[][])", Some(&nested)).await?,
        nested
    );
    Ok(())
}

async fn transactions(connection: &mut dyn Connection) -> Result<()> {
    connection.execute("BEGIN", &[]).await?;
    connection
        .execute("INSERT INTO sample VALUES (10, 'rollback')", &[])
        .await?;
    connection.execute("ROLLBACK", &[]).await?;
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*) FROM sample WHERE id = 10",
            None
        )
        .await?,
        Value::I64(0)
    );
    connection.execute("BEGIN", &[]).await?;
    connection
        .execute("INSERT INTO sample VALUES (11, 'commit')", &[])
        .await?;
    connection.execute("SAVEPOINT point", &[]).await?;
    connection
        .execute("DELETE FROM sample WHERE id = 11", &[])
        .await?;
    connection
        .execute("ROLLBACK TO SAVEPOINT point", &[])
        .await?;
    connection.execute("COMMIT", &[]).await?;
    assert_eq!(
        scalar(
            connection,
            "SELECT count(*) FROM sample WHERE id = 11",
            None
        )
        .await?,
        Value::I64(1)
    );
    Ok(())
}

async fn metadata(connection: &mut dyn Connection) -> Result<()> {
    connection.execute("CREATE TABLE parent (b INTEGER NOT NULL, a INTEGER NOT NULL, label TEXT DEFAULT 'default', PRIMARY KEY (b, a))", &[]).await?;
    connection.execute("CREATE TABLE child (id INTEGER PRIMARY KEY, b INTEGER, a INTEGER, CONSTRAINT child_parent FOREIGN KEY (b, a) REFERENCES parent(b, a))", &[]).await?;
    connection
        .execute("CREATE UNIQUE INDEX parent_label ON parent(label)", &[])
        .await?;
    connection
        .execute("CREATE INDEX parent_reverse ON parent(a, b)", &[])
        .await?;
    connection
        .execute(
            "CREATE VIEW parent_view AS SELECT b, a, label FROM parent",
            &[],
        )
        .await?;
    connection
        .execute("CREATE TABLE pattern_1 (one INTEGER)", &[])
        .await?;
    connection
        .execute("CREATE TABLE patternA1 (two INTEGER)", &[])
        .await?;
    let metadata = connection.metadata().await?;
    let catalog = metadata.current_catalog().context("current catalog")?;
    assert_eq!(catalog.name(), "jdbc_test");
    assert!(catalog.current());
    assert!(
        metadata
            .catalogs()
            .iter()
            .any(|catalog| catalog.name() == "postgres" && !catalog.current())
    );
    let schema = catalog.current_schema().context("current schema")?;
    assert_eq!(schema.name(), "public");
    assert!(schema.current());
    assert!(
        catalog
            .schemas()
            .iter()
            .any(|schema| schema.name() == "information_schema" && !schema.current())
    );
    parent_metadata(schema)?;
    child_and_view_metadata(schema)?;
    connection.execute("CREATE SCHEMA secondary", &[]).await?;
    connection
        .execute("SET search_path TO secondary", &[])
        .await?;
    connection
        .execute("CREATE TABLE other (id INTEGER)", &[])
        .await?;
    let metadata = connection.metadata().await?;
    let schema = metadata
        .current_catalog_schema()
        .context("secondary schema")?;
    assert_eq!(schema.name(), "secondary");
    assert!(schema.get("other").is_some());
    assert!(schema.get("parent").is_none());
    assert!(schema.views().is_empty());
    connection.execute("SET search_path TO public", &[]).await?;
    Ok(())
}

fn parent_metadata(schema: &rsql_driver::Schema) -> Result<()> {
    let parent = schema.get("parent").context("parent table")?;
    ensure!(
        parent
            .columns()
            .iter()
            .map(|column| column.name())
            .collect::<Vec<_>>()
            == ["b", "a", "label"],
        "unexpected parent columns or column order"
    );
    ensure!(
        parent.columns().first().context("column")?.not_null(),
        "the first parent column should be NOT NULL"
    );
    let b = parent.get_column("b").context("b column")?;
    ensure!(b.data_type() == "int4", "b should have type int4");
    ensure!(b.default().is_none(), "b should have no default");
    let label = parent.get_column("label").context("label column")?;
    ensure!(label.data_type() == "text", "label should have type text");
    ensure!(!label.not_null(), "label should be nullable");
    ensure!(
        label.default() == Some("'default'::text"),
        "unexpected label default"
    );
    let primary = parent.primary_key().context("primary key")?;
    ensure!(
        primary.name() == "parent_pkey",
        "unexpected primary key name"
    );
    ensure!(
        !primary.inferred(),
        "the primary key should not be inferred"
    );
    ensure!(
        primary.columns() == ["b", "a"],
        "unexpected primary key columns or column order"
    );
    ensure!(
        parent
            .indexes()
            .iter()
            .any(|index| index.name() == "parent_label" && index.unique()),
        "missing unique parent_label index"
    );
    ensure!(
        parent
            .get_index("parent_label")
            .context("unique index")?
            .columns()
            == ["label"],
        "unexpected unique index columns"
    );
    let index = parent
        .get_index("parent_reverse")
        .context("composite index")?;
    ensure!(
        index.columns() == ["a", "b"],
        "unexpected composite index columns or column order"
    );
    ensure!(!index.unique(), "the composite index should not be unique");
    Ok(())
}

fn child_and_view_metadata(schema: &rsql_driver::Schema) -> Result<()> {
    let child = schema.get("child").context("child table")?;
    let foreign = child
        .foreign_keys()
        .into_iter()
        .find(|key| key.name() == "child_parent")
        .context("foreign key")?;
    ensure!(
        foreign.columns() == ["b", "a"],
        "unexpected foreign key columns or column order"
    );
    ensure!(
        foreign.referenced_table() == "parent",
        "the foreign key should reference parent"
    );
    ensure!(
        foreign.referenced_columns() == ["b", "a"],
        "unexpected referenced columns or column order"
    );
    ensure!(
        !foreign.inferred(),
        "the foreign key should not be inferred"
    );
    ensure!(
        schema
            .get_view("parent_view")
            .context("view")?
            .columns()
            .len()
            == 3,
        "parent_view should contain three columns"
    );
    let view = schema.get_view("parent_view").context("view")?;
    ensure!(view.name() == "parent_view", "unexpected view name");
    ensure!(
        view.columns()
            .iter()
            .map(|column| (
                column.name(),
                column.data_type(),
                column.not_null(),
                column.default()
            ))
            .collect::<Vec<_>>()
            == [
                ("b", "int4", false, None),
                ("a", "int4", false, None),
                ("label", "text", false, None)
            ],
        "unexpected view column metadata"
    );
    ensure!(schema.get("parent_view").is_none(), "a view is not a table");
    ensure!(schema.get_view("parent").is_none(), "a table is not a view");
    let pattern = schema
        .get("pattern_1")
        .context("literal underscore table")?;
    ensure!(
        pattern.columns().len() == 1,
        "pattern_1 should have one column"
    );
    ensure!(
        pattern.primary_key().is_none(),
        "pattern_1 should have no primary key"
    );
    ensure!(
        pattern.indexes().is_empty(),
        "pattern_1 should have no indexes"
    );
    ensure!(
        pattern.foreign_keys().is_empty(),
        "pattern_1 should have no foreign keys"
    );
    ensure!(
        pattern.columns().first().context("pattern column")?.name() == "one",
        "the underscore in pattern_1 should match literally"
    );
    Ok(())
}

async fn errors(connection: &mut dyn Connection) -> Result<()> {
    assert!(
        connection
            .query("SELECT missing FROM sample", &[])
            .await
            .is_err()
    );
    assert!(
        connection
            .execute("INSERT INTO sample VALUES (1, 'duplicate')", &[])
            .await
            .is_err()
    );
    assert!(
        connection
            .query("SELECT CAST(? AS INTEGER)", &[&"not an integer"])
            .await
            .is_err()
    );
    assert!(
        connection
            .query("SELECT CAST(? AS INTEGER)", &[])
            .await
            .is_err()
    );
    assert!(connection.query("SELECT 1", &[&1_i32]).await.is_err());
    assert_eq!(scalar(connection, "SELECT 42", None).await?, Value::I32(42));
    Ok(())
}
