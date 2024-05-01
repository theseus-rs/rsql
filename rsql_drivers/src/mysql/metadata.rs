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
             WHERE table_schema = DATABASE()
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
            SELECT DISTINCT table_name, index_name
              FROM information_schema.statistics
             WHERE table_schema = DATABASE()
             ORDER BY table_name, index_name
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

#[cfg(target_os = "linux")]
#[cfg(test)]
mod test {
    use crate::{Connection, DriverManager};
    use testcontainers::runners::AsyncRunner;

    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let mysql_image =
            testcontainers::RunnableImage::from(testcontainers_modules::mysql::Mysql::default());
        let container = mysql_image.start().await;
        let port = container.get_host_port_ipv4(3306).await;

        let database_url = &format!("mysql://root@127.0.0.1:{port}/mysql");
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
        assert_eq!(contacts_indexes, vec!["PRIMARY"]);

        let user_table = database.get("users").unwrap();
        let user_indexes = user_table
            .indexes()
            .iter()
            .map(|index| index.name())
            .collect::<Vec<_>>();
        assert_eq!(user_indexes, vec!["PRIMARY"]);

        Ok(())
    }
}
