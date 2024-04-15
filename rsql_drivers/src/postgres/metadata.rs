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
            SELECT table_name
              FROM information_schema.tables
             WHERE table_catalog = current_database()
               AND table_schema = 'public'
             ORDER BY table_name
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
            SELECT ist.table_name, i.relname AS index_name
              FROM pg_class t,
                   pg_class i,
                   pg_index ix,
                   pg_attribute a,
                   information_schema.tables ist
             WHERE t.oid = ix.indrelid
               AND i.oid = ix.indexrelid
               AND a.attrelid = t.oid
               AND a.attnum = ANY(ix.indkey)
               AND t.relkind = 'r'
               AND ist.table_name = t.relname
               AND ist.table_schema = current_schema()
             ORDER BY ist.table_name, index_name
        "#};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.get(0) {
            Some(name) => name.to_string(),
            None => continue,
        };
        let index_name = match row.get(1) {
            Some(name) => name.to_string(),
            None => continue,
        };
        if let Some(table) = database.get_mut(table_name) {
            let index = Index::new(index_name, vec![], false);
            table.add_index(index);
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::DriverManager;

    const DATABASE_URL: &str = "postgres://?embedded=true";

    #[tokio::test]
    async fn test_schema() -> anyhow::Result<()> {
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(DATABASE_URL).await?;

        let _ = connection
            .execute("CREATE TABLE contacts (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, email VARCHAR(20))")
            .await?;

        let metadata = connection.metadata().await?;
        let database = metadata.current_database().unwrap();
        let tables = database.tables().iter().map(|table| table.name()).collect::<Vec<_>>();
        assert_eq!(tables, vec!["contacts", "users"]);

        let contacts_table = database.get("contacts").unwrap();
        let contacts_indexes = contacts_table.indexes().iter().map(|index| index.name()).collect::<Vec<_>>();
        assert_eq!(contacts_indexes, vec!["contacts_pkey"]);

        let user_table = database.get("users").unwrap();
        let user_indexes = user_table.indexes().iter().map(|index| index.name()).collect::<Vec<_>>();
        assert_eq!(user_indexes, vec!["users_pkey"]);

        connection.close().await?;
        Ok(())
    }
}
