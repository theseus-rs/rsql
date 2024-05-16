use crate::{Column, Connection, Database, Index, Metadata, Result, Table, Value};
use indoc::indoc;

pub(crate) async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let mut metadata = Metadata::default();

    retrieve_databases(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_databases(
    connection: &mut dyn Connection,
    metadata: &mut Metadata,
) -> Result<()> {
    let databases = vec![Database::new("default")];

    for mut database in databases {
        retrieve_tables(connection, &mut database).await?;
        retrieve_indexes(connection, &mut database).await?;
        metadata.add(database);
    }

    Ok(())
}

async fn retrieve_tables(connection: &mut dyn Connection, database: &mut Database) -> Result<()> {
    let sql = indoc! { r#"
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
                AND table_schema = 'public'
            ORDER BY
                table_name,
                ordinal_position
        "#};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.get(0) {
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
                    format!("{}({})", value, character_maximum_length)
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
        if let Some(table) = database.get_mut(&table_name) {
            table.add_column(column);
        } else {
            let mut table = Table::new(table_name);
            table.add_column(column);
            database.add(table);
        }
    }

    Ok(())
}

async fn retrieve_indexes(connection: &mut dyn Connection, database: &mut Database) -> Result<()> {
    let sql = indoc! {r#"
            SELECT
                ist.table_name,
                i.relname AS index_name,
                a.attname AS column_name,
                ix.indisunique AS unique,
                indisprimary AS primary_key
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
                index_name
        "#};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.get(0) {
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
        let primary_key = match row.get(4) {
            Some(Value::Bool(value)) => *value,
            _ => continue,
        };

        let table = match database.get_mut(table_name) {
            Some(table) => table,
            None => continue,
        };

        if let Some(index) = table.get_index_mut(&index_name) {
            index.add_column(column_name);
        } else {
            let index = Index::new(index_name, vec![column_name.clone()], primary_key, unique);
            table.add_index(index);
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[cfg(test)]
mod test {
    use crate::DriverManager;

    const DATABASE_URL: &str = "postgresql://?embedded=true";

    #[tokio::test]
    async fn test_schema() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute(
                r#"
                    CREATE TABLE contacts (
                        id INT4 NOT NULL PRIMARY KEY,
                        email VARCHAR(20) NULL UNIQUE
                    )
                "#,
            )
            .await?;
        let _ = connection
            .execute(
                r#"
                    CREATE TABLE users (
                        id INT4 NOT NULL PRIMARY KEY,
                        email VARCHAR(20) NULL UNIQUE
                    )
                "#,
            )
            .await?;

        let metadata = connection.metadata().await?;
        let database = metadata.current_database().unwrap();
        assert_eq!(database.tables().len(), 2);

        let contacts_table = database.get("contacts").unwrap();
        assert_eq!(contacts_table.name(), "contacts");
        assert_eq!(contacts_table.columns().len(), 2);
        let id_column = contacts_table.get_column("id").unwrap();
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "int4");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = contacts_table.get_column("email").unwrap();
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "varchar(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(contacts_table.indexes().len(), 2);
        let primary_key_index = contacts_table.get_index("contacts_pkey").unwrap();
        assert_eq!(primary_key_index.name(), "contacts_pkey");
        assert_eq!(primary_key_index.columns(), ["id"]);
        assert!(primary_key_index.unique());
        let email_index = contacts_table.get_index("contacts_email_key").unwrap();
        assert_eq!(email_index.name(), "contacts_email_key");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());

        let users_table = database.get("users").unwrap();
        assert_eq!(users_table.name(), "users");
        assert_eq!(users_table.columns().len(), 2);
        let id_column = users_table.get_column("id").unwrap();
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "int4");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = users_table.get_column("email").unwrap();
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "varchar(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(users_table.indexes().len(), 2);
        let primary_key_index = users_table.get_index("users_pkey").unwrap();
        assert_eq!(primary_key_index.name(), "users_pkey");
        assert_eq!(primary_key_index.columns(), ["id"]);
        assert!(primary_key_index.unique());
        let email_index = users_table.get_index("users_email_key").unwrap();
        assert_eq!(email_index.name(), "users_email_key");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());

        connection.close().await?;
        Ok(())
    }
}
