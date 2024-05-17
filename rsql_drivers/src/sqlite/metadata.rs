use crate::{Column, Connection, Database, Index, Metadata, Result, Table};
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
        let table_name = match row.get(0) {
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
            Some(value) => value.to_string() == "1",
            None => continue,
        };

        let table = match database.get_mut(table_name) {
            Some(table) => table,
            None => continue,
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
    use crate::DriverManager;

    const DATABASE_URL: &str = "sqlite://?memory=true";

    #[tokio::test]
    async fn test_schema() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute(
                r#"
                    CREATE TABLE contacts (
                        id INTEGER NOT NULL PRIMARY KEY,
                        email VARCHAR(20) NULL UNIQUE
                    )
                "#,
            )
            .await?;
        let _ = connection
            .execute(
                r#"
                    CREATE TABLE users (
                        id INTEGER NOT NULL PRIMARY KEY,
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
        assert_eq!(id_column.data_type(), "INTEGER");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = contacts_table.get_column("email").unwrap();
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "VARCHAR(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(contacts_table.indexes().len(), 1);
        let email_index = contacts_table
            .get_index("sqlite_autoindex_contacts_1")
            .unwrap();
        assert_eq!(email_index.name(), "sqlite_autoindex_contacts_1");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());

        let users_table = database.get("users").unwrap();
        assert_eq!(users_table.name(), "users");
        assert_eq!(users_table.columns().len(), 2);
        let id_column = users_table.get_column("id").unwrap();
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "INTEGER");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = users_table.get_column("email").unwrap();
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "VARCHAR(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        assert_eq!(users_table.indexes().len(), 1);
        let email_index = users_table.get_index("sqlite_autoindex_users_1").unwrap();
        assert_eq!(email_index.name(), "sqlite_autoindex_users_1");
        assert_eq!(email_index.columns(), ["email"]);
        assert!(email_index.unique());

        connection.close().await?;
        Ok(())
    }
}
