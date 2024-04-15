use indoc::indoc;
use crate::{Connection, Database, Index, Metadata, Result, Table};

pub(crate) async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let mut metadata = Metadata::default();

    retrieve_databases(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_databases(
    connection: &mut dyn Connection,
    metadata: &mut Metadata) -> Result<()> {
    let databases = vec![Database::new("default")];

    for mut database in databases {
        retrieve_tables(connection, &mut database).await?;
        retrieve_indexes(connection, &mut database).await?;
        metadata.add(database);
    }

    Ok(())
}

async fn retrieve_tables(
    connection: &mut dyn Connection,
    database: &mut Database) -> Result<()> {
    let sql = indoc! { r#"
            SELECT name
              FROM sqlite_master
             WHERE type='table'
             ORDER BY name
        "#};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        if let Some(data) = row.get(0) {
            let table = Table::new(data.to_string());
            database.add(table);
        }
    }

    Ok(())
}

async fn retrieve_indexes(
    connection: &mut dyn Connection,
    database: &mut Database) -> Result<()> {
    let sql = indoc! {r#"
            SELECT tbl_name as table_name, name as index_name
              FROM sqlite_master
             WHERE type = 'index'
             ORDER BY table_name, index_name
        "#};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = row.get(0).unwrap();
        if let Some(table) = database.get_mut(table_name.to_string()) {
            let index_name = row.get(1).unwrap();
            let index = Index::new(index_name.to_string(), vec![], false);
            table.add_index(index);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::DriverManager;

    const DATABASE_URL: &str = "libsql://?memory=true";

    #[tokio::test]
    async fn test_schema() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20) UNIQUE)")
            .await?;

        let metadata = connection.metadata().await?;
        let database = metadata.current_database().unwrap();
        let tables = database.tables().iter().map(|table| table.name()).collect::<Vec<_>>(;
        assert_eq!(tables, vec!["contacts", "users"]);

        let contacts_table = database.get("contacts").unwrap();
        let contacts_indexes = contacts_table.indexes().iter().map(|index| index.name()).collect::<Vec<_>>();
        assert_eq!(contacts_indexes, vec!["sqlite_autoindex_contacts_1"]);

        let user_table = database.get("users").unwrap();
        let user_indexes = user_table.indexes().iter().map(|index| index.name()).collect::<Vec<_>>();
        assert_eq!(user_indexes, vec!["sqlite_autoindex_users_1"]);

        connection.close().await?;
        Ok(())
    }
}