use indoc::indoc;
use regex::Regex;
use rsql_driver::Error::IoError;
use rsql_driver::{Catalog, Column, Connection, Index, Metadata, Result, Schema, Table, Value};

pub(crate) async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
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
                data_type,
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
            Some(value) => value.to_string(),
            None => continue,
        };
        let not_null = match row.get(3) {
            Some(value) => value.to_string() == "NO",
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
                table_name,
                index_name,
                sql,
                is_unique
            FROM
                duckdb_indexes()
            ORDER BY
                table_name,
                index_name
        "};
    let mut query_result = connection.query(sql).await?;
    let regex = Regex::new(r"\((.*?)\)").map_err(|error| IoError(error.to_string()))?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let index_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let columns = match row.get(2) {
            Some(value) => {
                let sql = value.to_string();
                let mut columns: Vec<String> = Vec::new();

                for captures in regex.captures_iter(sql.as_str()) {
                    let column_string = &captures[1];
                    for column in column_string.split(", ") {
                        columns.push(column.to_string());
                    }
                }

                columns
            }
            None => continue,
        };
        let unique = match row.get(3) {
            Some(Value::Bool(value)) => *value,
            _ => continue,
        };

        if let Some(table) = schema.get_mut(table_name) {
            let index = Index::new(index_name, columns, unique);
            table.add_index(index);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::Driver;

    const DATABASE_URL: &str = "duckdb://";

    #[tokio::test]
    async fn test_schema() -> Result<()> {
        let driver = crate::Driver;
        let mut connection = driver.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;
        let _ = connection
            .execute("CREATE UNIQUE INDEX contacts_email_idx ON contacts (email)")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;
        let _ = connection
            .execute("CREATE UNIQUE INDEX users_email_idx ON users (email)")
            .await?;

        let metadata = connection.metadata().await?;
        assert_eq!(metadata.catalogs().len(), 3);
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
        assert_eq!(email_column.data_type(), "VARCHAR");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(contacts_table.indexes().len(), 1);
        let email_index = contacts_table
            .get_index("contacts_email_idx")
            .expect("index");
        assert_eq!(email_index.name(), "contacts_email_idx");
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
        assert_eq!(email_column.data_type(), "VARCHAR");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(users_table.indexes().len(), 1);
        let email_index = users_table.get_index("users_email_idx").expect("index");
        assert_eq!(email_index.name(), "users_email_idx");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());

        connection.close().await?;
        Ok(())
    }
}
