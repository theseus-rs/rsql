use async_trait::async_trait;
use file_type::FileType;
use rsql_driver::{Error, Result};
use rsql_driver_jdbc::{Connection, Options};

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "h2"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let options = options(url)?;
        Ok(Box::new(Connection::new(url, options).await?))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

fn options(url: &str) -> Result<Options> {
    let database = url
        .strip_prefix("h2:")
        .ok_or_else(|| Error::InvalidUrl("Expected h2:<database>".into()))?;
    let (database, query) = database.split_once('?').unwrap_or((database, ""));
    let database = database.strip_prefix("//").unwrap_or(database);
    let database = if database.is_empty() {
        "mem:"
    } else {
        database
    };
    #[cfg(not(target_family = "wasm"))]
    // Declare the pinned driver first so it takes precedence over additional dependencies.
    let mut options = Options::parse(&format!(
        "jdbc:h2:{database}?dependency=com.h2database:h2:{}&{query}",
        crate::H2_VERSION
    ))?;
    #[cfg(target_family = "wasm")]
    let mut options = Options::parse(&format!("jdbc:h2:{database}?{query}"))?;
    options.driver = Some("org.h2.Driver".into());
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls() -> Result<()> {
        for url in ["h2:", "h2://"] {
            let options = options(url)?;
            assert_eq!(options.url, "jdbc:h2:mem:");
            assert_eq!(options.driver.as_deref(), Some("org.h2.Driver"));
            #[cfg(not(target_family = "wasm"))]
            assert_eq!(
                options
                    .dependencies
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                [format!("com.h2database:h2:{}", crate::H2_VERSION)]
            );
        }
        for database in [
            "mem:test;DB_CLOSE_DELAY=-1",
            "./test",
            "/tmp/test",
            "tcp://localhost/~/test",
        ] {
            assert_eq!(
                options(&format!("h2:{database}"))?.url,
                format!("jdbc:h2:{database}")
            );
        }
        assert_eq!(
            options("h2://?driver=other.Driver")?.driver.as_deref(),
            Some("org.h2.Driver")
        );
        assert!(options("jdbc:h2:mem:").is_err());
        let options = options(
            "h2:mem:test;MODE=PostgreSQL?dependency=example:extension:1.0&classpath=local.jar",
        )?;
        assert_eq!(options.url, "jdbc:h2:mem:test;MODE=PostgreSQL");
        assert_eq!(
            options.dependencies.len(),
            if cfg!(target_family = "wasm") { 1 } else { 2 }
        );
        assert_eq!(options.classpath, [std::path::PathBuf::from("local.jar")]);
        Ok(())
    }
}
