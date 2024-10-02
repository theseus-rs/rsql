use crate::error::Result;
use crate::postgresql::driver::Connection;
use async_trait::async_trait;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "redshift"
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
    use crate::{DriverManager, Value};
    use testcontainers::runners::AsyncRunner;

    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        // Skip tests on GitHub Actions for non-Linux platforms; the test containers fail to run.
        if std::env::var("GITHUB_ACTIONS").is_ok() && !cfg!(target_os = "linux") {
            return Ok(());
        }

        let postgres_image = testcontainers::ContainerRequest::from(
            testcontainers_modules::postgres::Postgres::default(),
        );
        let container = postgres_image.start().await?;
        let port = container.get_host_port_ipv4(5432).await?;

        let database_url = format!("redshift://postgres:postgres@localhost:{port}/postgres");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        let mut query_result = connection.query("SELECT 'foo'::TEXT").await?;
        let row = query_result.next().await.expect("no row");
        let value = row.first().expect("no value");

        assert_eq!(*value, Value::String("foo".to_string()));

        container.stop().await?;
        container.rm().await?;
        Ok(())
    }
}
