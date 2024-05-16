use crate::{Connection, Database, Index, Metadata, Result, Table};
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
            SELECT table_name
              FROM information_schema.tables
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

async fn retrieve_indexes(connection: &mut dyn Connection, database: &mut Database) -> Result<()> {
    let sql = indoc! {r#"
            SELECT tables.name, indexes.name
              FROM sys.indexes
             INNER JOIN sys.tables ON indexes.object_id = tables.object_id
             WHERE tables.is_ms_shipped = 0
             ORDER BY tables.name, indexes.name
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
            let index = Index::new(index_name, vec![], false, false);
            table.add_index(index);
        }
    }

    Ok(())
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod test {
    use crate::{Connection, DriverManager};
    use testcontainers::runners::AsyncRunner;
    use testcontainers::RunnableImage;
    use testcontainers_modules::mssql_server::MssqlServer;

    const PASSWORD: &str = "Password42!";

    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let sqlserver_image =
            RunnableImage::from(MssqlServer::default().with_sa_password(PASSWORD));
        let container = sqlserver_image.start().await;
        let port = container.get_host_port_ipv4(1433).await;
        let database_url =
            &format!("sqlserver://sa:{PASSWORD}@127.0.0.1:{port}?TrustServerCertificate=true");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        test_schema(&mut *connection).await?;

        Ok(())
    }

    async fn test_schema(connection: &mut dyn Connection) -> anyhow::Result<()> {
        let _ = connection
            .execute("CREATE TABLE contacts (id INT PRIMARY KEY, email VARCHAR(20))")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(20))")
            .await?;

        let metadata = connection.metadata().await?;
        let database = metadata.current_database().unwrap();
        let tables = database
            .tables()
            .iter()
            .map(|table| table.name())
            .collect::<Vec<_>>();
        assert!(tables.contains(&"contacts"));
        assert!(tables.contains(&"users"));

        let contacts_table = database.get("contacts").unwrap();
        let contacts_indexes = contacts_table
            .indexes()
            .iter()
            .map(|index| index.name())
            .collect::<Vec<_>>();
        assert!(contacts_indexes[0].contains(&"PK__contacts__".to_string()));

        let user_table = database.get("users").unwrap();
        let user_indexes = user_table
            .indexes()
            .iter()
            .map(|index| index.name())
            .collect::<Vec<_>>();
        assert!(user_indexes[0].contains(&"PK__users__".to_string()));

        Ok(())
    }
}
