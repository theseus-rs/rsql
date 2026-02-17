use indoc::indoc;
use rsql_driver::{
    Catalog, Column, Connection, ForeignKey, Index, Metadata, PrimaryKey, Result, Schema, Table,
    View,
};

pub(crate) async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let mut metadata = Metadata::with_dialect(connection.dialect());

    retrieve_catalogs(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_catalogs(connection: &mut dyn Connection, metadata: &mut Metadata) -> Result<()> {
    let mut catalogs = vec![Catalog::new("default", true)];
    catalogs.sort_by_key(|catalog| catalog.name().to_ascii_lowercase());

    for mut catalog in catalogs {
        retrieve_schemas(connection, &mut catalog).await?;
        metadata.add(catalog);
    }

    Ok(())
}

async fn retrieve_schemas(connection: &mut dyn Connection, catalog: &mut Catalog) -> Result<()> {
    let mut schemas = vec![];
    let sql = indoc! { "SELECT name FROM pragma_database_list ORDER BY name" };
    let mut query_result = connection.query(sql, &[]).await?;

    while let Some(row) = query_result.next().await {
        let database_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let current = database_name == "main";
        let schema = Schema::new(database_name, current);
        schemas.push(schema);
    }

    for mut schema in schemas {
        if schema.current() {
            retrieve_tables(connection, &mut schema).await?;
            retrieve_views(connection, &mut schema).await?;
            retrieve_indexes(connection, &mut schema).await?;
            retrieve_primary_keys(connection, &mut schema).await?;
            retrieve_foreign_keys(connection, &mut schema).await?;
        }
        catalog.add(schema);
    }

    Ok(())
}

async fn retrieve_views(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! { r#"
            SELECT
                m.name AS view_name,
                p.name AS column_name,
                p.type AS column_type,
                p."notnull" AS not_null,
                p.dflt_value AS default_value
            FROM
                sqlite_master m
                LEFT OUTER JOIN pragma_table_info((m.name)) p ON m.name <> p.name
            WHERE
                m.type = 'view'
            ORDER BY
                view_name,
                p.cid,
                column_name
        "#};
    let mut query_result = connection.query(sql, &[]).await?;

    while let Some(row) = query_result.next().await {
        let view_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_type = match row.get(2) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let not_null = match row.get(3) {
            Some(value) => value.to_string() == "1",
            None => continue,
        };
        let default_value = match row.get(4) {
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
        if let Some(view) = schema.get_view_mut(&view_name) {
            view.add_column(column);
        } else {
            let mut view = View::new(view_name);
            view.add_column(column);
            schema.add_view(view);
        }
    }

    Ok(())
}

async fn retrieve_tables(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! { r#"
            SELECT
                m.name AS table_name,
                p.name AS column_name,
                p.type AS column_type,
                p."notnull" AS not_null,
                p.dflt_value AS default_value
            FROM
                sqlite_master m
                LEFT OUTER JOIN pragma_table_info((m.name)) p ON m.name <> p.name
            WHERE
                m.type = 'table'
            ORDER BY
                table_name,
                p.cid,
                column_name
        "#};
    let mut query_result = connection.query(sql, &[]).await?;

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
            Some(value) => value.to_string(),
            None => continue,
        };
        let not_null = match row.get(3) {
            Some(value) => value.to_string() == "1",
            None => continue,
        };
        let default_value = match row.get(4) {
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
                m.tbl_name AS table_name,
                il.name AS index_name,
                ii.name AS column_name,
                il.[unique]
            FROM
                sqlite_master AS m,
                pragma_index_list(m.name) AS il,
                pragma_index_info(il.name) AS ii
            WHERE
                m.type = 'table'
            GROUP BY
                m.tbl_name,
                il.name,
                ii.name
            ORDER BY
                index_name,
                il.seq,
                ii.seqno
        "};
    let mut query_result = connection.query(sql, &[]).await?;

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
            Some(value) => value.to_string() == "1",
            None => continue,
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
    let table_names: Vec<String> = schema
        .tables()
        .iter()
        .map(|t| t.name().to_string())
        .collect();

    for table_name in table_names {
        let sql = "SELECT name, pk FROM pragma_table_info(?) WHERE pk > 0 ORDER BY pk";
        let mut query_result = connection.query(sql, &[&table_name]).await?;
        let mut pk_columns = Vec::new();

        while let Some(row) = query_result.next().await {
            let column_name = match row.first() {
                Some(value) => value.to_string(),
                None => continue,
            };
            pk_columns.push(column_name);
        }

        if !pk_columns.is_empty()
            && let Some(table) = schema.get_mut(&table_name)
        {
            let pk_name = format!("{table_name}_pkey");
            let pk = PrimaryKey::new(pk_name, pk_columns, false);
            table.set_primary_key(pk);
        }
    }

    Ok(())
}

async fn retrieve_foreign_keys(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let table_names: Vec<String> = schema
        .tables()
        .iter()
        .map(|t| t.name().to_string())
        .collect();

    for table_name in table_names {
        let sql = "SELECT * FROM pragma_foreign_key_list(?)";
        let mut query_result = connection.query(sql, &[&table_name]).await?;

        while let Some(row) = query_result.next().await {
            let id = match row.first() {
                Some(value) => value.to_string(),
                None => continue,
            };
            let referenced_table = match row.get(2) {
                Some(value) => value.to_string(),
                None => continue,
            };
            let from_column = match row.get(3) {
                Some(value) => value.to_string(),
                None => continue,
            };
            let to_column = match row.get(4) {
                Some(value) => value.to_string(),
                None => continue,
            };

            let fk_name = format!("{table_name}_{from_column}_fk_{id}");

            if let Some(table) = schema.get_mut(&table_name) {
                let fk = ForeignKey::new(
                    fk_name,
                    vec![from_column],
                    referenced_table,
                    vec![to_column],
                    false,
                );
                table.add_foreign_key(fk);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::Driver;

    const DATABASE_URL: &str = "libsql://";

    #[tokio::test]
    async fn test_schema() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let _ = connection
            .execute(
                r"
                    CREATE TABLE contacts (
                        id INTEGER NOT NULL PRIMARY KEY,
                        email VARCHAR(20) NULL UNIQUE
                    )
                ",
                &[],
            )
            .await?;
        let _ = connection
            .execute(
                r"
                    CREATE TABLE users (
                        id INTEGER NOT NULL PRIMARY KEY,
                        email VARCHAR(20) NULL UNIQUE
                    )
                ",
                &[],
            )
            .await?;

        let metadata = connection.metadata().await?;
        assert_eq!(metadata.catalogs().len(), 1);
        let catalog = metadata.current_catalog().expect("catalog");
        assert_eq!(catalog.schemas().len(), 1);
        let schema = catalog.current_schema().expect("schema");
        assert_eq!(schema.tables().len(), 2);

        let contacts_table = schema.get("contacts").expect("contacts table");
        assert_eq!(contacts_table.name(), "contacts");
        assert_eq!(contacts_table.columns().len(), 2);
        let id_column = contacts_table.get_column("id").expect("id column");
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "INTEGER");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = contacts_table.get_column("email").expect("email column");
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "VARCHAR(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(contacts_table.indexes().len(), 1);
        let email_index = contacts_table
            .get_index("sqlite_autoindex_contacts_1")
            .expect("index");
        assert_eq!(email_index.name(), "sqlite_autoindex_contacts_1");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());
        let pk = contacts_table.primary_key().expect("primary key");
        assert_eq!(pk.name(), "contacts_pkey");
        assert_eq!(pk.columns(), &["id".to_string()]);
        assert!(!pk.inferred());

        let users_table = schema.get("users").expect("users table");
        assert_eq!(users_table.name(), "users");
        assert_eq!(users_table.columns().len(), 2);
        let id_column = users_table.get_column("id").expect("id column");
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "INTEGER");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = users_table.get_column("email").expect("email column");
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "VARCHAR(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(users_table.indexes().len(), 1);
        let email_index = users_table
            .get_index("sqlite_autoindex_users_1")
            .expect("index");
        assert_eq!(email_index.name(), "sqlite_autoindex_users_1");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());
        let pk = users_table.primary_key().expect("primary key");
        assert_eq!(pk.name(), "users_pkey");
        assert_eq!(pk.columns(), &["id".to_string()]);
        assert!(!pk.inferred());

        connection.close().await?;
        Ok(())
    }
}
