use indoc::indoc;
use rsql_driver::{Column, Connection, Index, Metadata, Result, Schema, Table};

/// Retrieves the metadata from the database.
///
/// # Errors
/// if an error occurs while retrieving the metadata.
pub async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let mut metadata = Metadata::with_dialect(connection.dialect());

    retrieve_schemas(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_schemas(connection: &mut dyn Connection, metadata: &mut Metadata) -> Result<()> {
    let mut schemas = vec![];
    let sql = indoc! { "SELECT name FROM pragma_database_list ORDER BY name"};
    let mut query_result = connection.query(sql).await?;

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
            retrieve_indexes(connection, &mut schema).await?;
        }
        metadata.add(schema);
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

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::Driver;

    const DATABASE_URL: &str = "sqlite://";

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
            )
            .await?;

        let metadata = connection.metadata().await?;
        let schema = metadata.current_schema().expect("schema");
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

        connection.close().await?;
        Ok(())
    }
}
