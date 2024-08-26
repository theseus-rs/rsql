use crate::error::Result;
use crate::mysql::driver::Connection;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "mariadb"
    }

    async fn connect(
        &self,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let connection = Connection::new(url, password).await?;
        Ok(Box::new(connection))
    }
}

#[cfg(test)]
mod test {
    use crate::{Connection, DriverManager};
    use testcontainers::runners::AsyncRunner;

    #[allow(dead_code)]
    // #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let mysql_image = testcontainers::ContainerRequest::from(
            testcontainers_modules::mariadb::Mariadb::default(),
        );
        let container = mysql_image.start().await?;
        let port = container.get_host_port_ipv4(3306).await?;

        let database_url = &format!("mariadb://root@127.0.0.1:{port}/test");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        test_schema(&mut *connection).await?;

        container.stop().await?;
        container.rm().await?;
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
        let schema = metadata.current_schema().expect("schema");
        let tables = schema
            .tables()
            .iter()
            .map(|table| table.name())
            .collect::<Vec<_>>();
        assert!(tables.contains(&"contacts"));
        assert!(tables.contains(&"users"));

        let contacts_table = schema.get("contacts").expect("contacts table");
        let contacts_indexes = contacts_table
            .indexes()
            .iter()
            .map(|index| index.name())
            .collect::<Vec<_>>();
        assert_eq!(contacts_indexes, vec!["PRIMARY"]);

        let user_table = schema.get("users").expect("users table");
        let user_indexes = user_table
            .indexes()
            .iter()
            .map(|index| index.name())
            .collect::<Vec<_>>();
        assert_eq!(user_indexes, vec!["PRIMARY"]);

        Ok(())
    }
}
