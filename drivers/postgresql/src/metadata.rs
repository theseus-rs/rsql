use indoc::indoc;
use rsql_driver::{
    Catalog, Column, Connection, ForeignKey, Index, Metadata, PrimaryKey, Result, Schema, Table,
    Value,
};

/// Retrieves the metadata from the database.
///
/// # Errors
/// if an error occurs while retrieving the metadata.
pub async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let mut metadata = Metadata::with_dialect(connection.dialect());

    retrieve_catalogs(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_catalogs(connection: &mut dyn Connection, metadata: &mut Metadata) -> Result<()> {
    let mut catalogs = vec![];
    let sql = indoc! { r"
        SELECT
            catalog_name,
            catalog_name = current_database() AS current_catalog
        FROM
            information_schema.schemata
        GROUP BY
            catalog_name
        ORDER BY
            catalog_name
    "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let catalog_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let current = matches!(row.get(1), Some(Value::Bool(true)));
        let catalog = Catalog::new(catalog_name, current);
        catalogs.push(catalog);
    }

    for mut catalog in catalogs {
        retrieve_schemas(connection, &mut catalog).await?;
        metadata.add(catalog);
    }

    Ok(())
}

async fn retrieve_schemas(connection: &mut dyn Connection, catalog: &mut Catalog) -> Result<()> {
    let mut schemas = vec![];
    let sql = indoc! { r"
        SELECT
            schema_name,
            schema_name = current_schema() AS current_schema
        FROM
            information_schema.schemata
        WHERE
            catalog_name = current_database()
        GROUP BY
            schema_name
        ORDER BY
            schema_name
    "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let schema_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let current = matches!(row.get(1), Some(Value::Bool(true)));
        let schema = Schema::new(schema_name, current);
        schemas.push(schema);
    }

    for mut schema in schemas {
        if schema.current() {
            retrieve_tables(connection, &mut schema).await?;
            retrieve_indexes(connection, &mut schema).await?;
            retrieve_primary_keys(connection, &mut schema).await?;
            retrieve_foreign_keys(connection, &mut schema).await?;
        }
        catalog.add(schema);
    }

    Ok(())
}

async fn retrieve_tables(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! { r"
            SELECT
                table_name,
                column_name,
                udt_name,
                character_maximum_length,
                is_nullable,
                column_default
            FROM
                information_schema.columns
            WHERE
                table_catalog = current_database()
                AND table_schema = current_schema()
            ORDER BY
                table_name,
                ordinal_position
        "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_type = match row.get(2) {
            Some(value) => {
                let character_maximum_length = row.get(3).unwrap_or(&Value::Null);

                if character_maximum_length.is_null() {
                    value.to_string()
                } else {
                    format!("{value}({character_maximum_length})")
                }
            }
            None => continue,
        };
        let not_null = match row.get(4) {
            Some(value) => value.to_string() == "NO",
            None => continue,
        };
        let default_value = match row.get(5) {
            Some(value) => {
                if value.is_null() {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            None => continue,
        };

        let column = Column::new(column_name, column_type, not_null, default_value);
        if let Some(table) = schema.get_mut(&table_name) {
            table.add_column(column);
        } else {
            let mut table = Table::new(table_name);
            table.add_column(column);
            schema.add(table);
        }
    }

    Ok(())
}

async fn retrieve_indexes(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! {r"
            SELECT
                ist.table_name,
                i.relname AS index_name,
                a.attname AS column_name,
                ix.indisunique AS unique
            FROM
                pg_class t
                JOIN pg_index ix ON ix.indrelid = t.oid
                JOIN pg_attribute a ON a.attrelid = t.oid
                JOIN pg_class i ON i.oid = ix.indexrelid
                JOIN information_schema.tables ist ON ist.table_name = t.relname
            WHERE
                a.attnum = ANY(ix.indkey)
                AND t.relkind = 'r'
                AND ist.table_schema = current_schema()
            ORDER BY
                ist.table_name,
                index_name,
                array_position(ix.indkey, a.attnum)
        "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let index_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(2) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let unique = match row.get(3) {
            Some(Value::Bool(value)) => *value,
            _ => continue,
        };
        let Some(table) = schema.get_mut(table_name) else {
            continue;
        };

        if let Some(index) = table.get_index_mut(&index_name) {
            index.add_column(column_name);
        } else {
            let index = Index::new(index_name, vec![column_name.clone()], unique);
            table.add_index(index);
        }
    }

    Ok(())
}

async fn retrieve_primary_keys(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! {r"
            SELECT
                kcu.table_name,
                tc.constraint_name,
                kcu.column_name
            FROM
                information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
            WHERE
                tc.constraint_type = 'PRIMARY KEY'
                AND tc.table_schema = current_schema()
            ORDER BY
                kcu.table_name,
                tc.constraint_name,
                kcu.ordinal_position
        "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let constraint_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(2) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let Some(table) = schema.get_mut(table_name) else {
            continue;
        };

        if let Some(pk) = table.primary_key() {
            // Multi-column PK: already captured by first row
            let _ = pk;
        } else {
            let pk = PrimaryKey::new(constraint_name, vec![column_name], false);
            table.set_primary_key(pk);
        }
    }

    Ok(())
}

async fn retrieve_foreign_keys(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! {r"
            SELECT
                kcu.table_name,
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name AS referenced_table_name,
                ccu.column_name AS referenced_column_name
            FROM
                information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage ccu
                    ON ccu.constraint_name = tc.constraint_name
                    AND ccu.table_schema = tc.table_schema
            WHERE
                tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = current_schema()
            ORDER BY
                kcu.table_name,
                tc.constraint_name,
                kcu.ordinal_position
        "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let constraint_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(2) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let referenced_table = match row.get(3) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let referenced_column = match row.get(4) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let Some(table) = schema.get_mut(table_name) else {
            continue;
        };

        if table.get_foreign_key(&constraint_name).is_some() {
            // Multi-column FK: columns already captured by first row
            continue;
        }

        let fk = ForeignKey::new(
            constraint_name,
            vec![column_name],
            referenced_table,
            vec![referenced_column],
            false,
        );
        table.add_foreign_key(fk);
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::Driver;

    const DATABASE_URL: &str = "postgresql://?embedded=true";

    #[tokio::test]
    async fn test_schema() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let _ = connection
            .execute(
                r"
                    CREATE TABLE contacts (
                        id INT4 NOT NULL PRIMARY KEY,
                        email VARCHAR(20) NULL UNIQUE
                    )
                ",
            )
            .await?;
        let _ = connection
            .execute(
                r"
                    CREATE TABLE users (
                        id INT4 NOT NULL PRIMARY KEY,
                        email VARCHAR(20) NULL UNIQUE
                    )
                ",
            )
            .await?;
        let _ = connection
            .execute("CREATE INDEX users_emails ON users (id, email)")
            .await?;

        let metadata = connection.metadata().await?;
        assert_eq!(metadata.catalogs().len(), 1);
        let catalog = metadata.current_catalog().expect("catalog");
        assert_eq!(catalog.schemas().len(), 4);
        let schema = catalog.current_schema().expect("schema");
        assert_eq!(schema.tables().len(), 2);

        let contacts_table = schema.get("contacts").expect("contacts table");
        assert_eq!(contacts_table.name(), "contacts");
        assert_eq!(contacts_table.columns().len(), 2);
        let id_column = contacts_table.get_column("id").expect("id column");
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "int4");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = contacts_table.get_column("email").expect("email column");
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "varchar(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(contacts_table.indexes().len(), 2);
        let primary_key_index = contacts_table.get_index("contacts_pkey").expect("index");
        assert_eq!(primary_key_index.name(), "contacts_pkey");
        assert_eq!(primary_key_index.columns(), ["id"]);
        assert!(primary_key_index.unique());
        let pk = contacts_table.primary_key().expect("primary key");
        assert_eq!(pk.name(), "contacts_pkey");
        assert_eq!(pk.columns(), &["id".to_string()]);
        assert!(!pk.inferred());
        let email_index = contacts_table
            .get_index("contacts_email_key")
            .expect("index");
        assert_eq!(email_index.name(), "contacts_email_key");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());

        let users_table = schema.get("users").expect("users table");
        assert_eq!(users_table.name(), "users");
        assert_eq!(users_table.columns().len(), 2);
        let id_column = users_table.get_column("id").expect("id column");
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "int4");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = users_table.get_column("email").expect("email column");
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "varchar(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(users_table.indexes().len(), 3);
        let primary_key_index = users_table.get_index("users_pkey").expect("index");
        assert_eq!(primary_key_index.name(), "users_pkey");
        assert_eq!(primary_key_index.columns(), ["id"]);
        assert!(primary_key_index.unique());
        let pk = users_table.primary_key().expect("primary key");
        assert_eq!(pk.name(), "users_pkey");
        assert_eq!(pk.columns(), &["id".to_string()]);
        assert!(!pk.inferred());
        let email_index = users_table.get_index("users_email_key").expect("index");
        assert_eq!(email_index.name(), "users_email_key");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());
        let users_emails_index = users_table.get_index("users_emails").expect("index");
        assert_eq!(users_emails_index.name(), "users_emails");
        assert_eq!(users_emails_index.columns(), ["id", "email"]);
        assert!(!users_emails_index.unique());

        connection.close().await?;
        Ok(())
    }
}
