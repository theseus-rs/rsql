use crate::connection::CachedMetadataConnection;
use crate::error::Result;
use crate::url::UrlExtension;
use crate::DriverManager;
use crate::Error::{DriverNotFound, IoError};
use async_trait::async_trait;
use file_type::FileType;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "file"
    }

    async fn connect(
        &self,
        url: String,
        _password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let parsed_url = Url::parse(url.as_str())?;
        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let file_type = FileType::try_from_file(&file_name)
            .await
            .map_err(|error| IoError(error.into()))?;
        let driver_manager = DriverManager::default();
        let driver = driver_manager.get_by_file_type(file_type);

        match driver {
            Some(driver) => {
                let scheme = format!("{}:", parsed_url.scheme());
                let url = url.strip_prefix(&scheme).unwrap_or(&url);
                let url = format!("{}:{url}", driver.identifier());
                let connection = driver_manager.connect(url.as_str()).await?;
                let connection = Box::new(CachedMetadataConnection::new(connection));
                Ok(connection)
            }
            None => Err(DriverNotFound(format!(
                "{file_name}: {:?}",
                file_type.media_types()
            ))),
        }
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::test::dataset_url;
    use crate::{DriverManager, Value};
    use indoc::indoc;

    #[tokio::test]
    async fn test_file_drivers() -> anyhow::Result<()> {
        let database_urls = vec![
            #[cfg(feature = "arrow")]
            (dataset_url("file", "users.arrow"), None),
            #[cfg(feature = "avro")]
            (dataset_url("file", "users.avro"), None),
            #[cfg(feature = "csv")]
            (dataset_url("file", "users.csv"), None),
            #[cfg(feature = "duckdb")]
            (dataset_url("file", "users.duckdb"), None),
            #[cfg(feature = "json")]
            (dataset_url("file", "users.json"), None),
            #[cfg(feature = "jsonl")]
            (dataset_url("file", "users.jsonl"), None),
            #[cfg(feature = "parquet")]
            (dataset_url("file", "users.parquet"), None),
            #[cfg(feature = "sqlite")]
            (dataset_url("file", "users.sqlite3"), None),
            #[cfg(feature = "tsv")]
            (dataset_url("file", "users.tsv"), None),
            #[cfg(feature = "xml")]
            (
                dataset_url("file", "users.xml"),
                Some(indoc! {r"
                    WITH cte_user AS (
                        SELECT unnest(data.user) FROM users
                    )
                    SELECT user.* FROM cte_user
                "}),
            ),
            #[cfg(feature = "yaml")]
            (dataset_url("file", "users.yaml"), None),
        ];
        for (database_url, sql) in database_urls {
            test_file_driver(database_url.as_str(), sql).await?;
        }
        Ok(())
    }

    async fn test_file_driver(database_url: &str, sql: Option<&str>) -> anyhow::Result<()> {
        let sql = sql.unwrap_or("SELECT id, name FROM users ORDER BY id");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url).await?;

        let mut query_result = connection.query(sql).await?;

        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
        );
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }
}
