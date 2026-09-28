use ristretto_resolver::ArtifactCoordinate;
use rsql_driver::{Error, Result};
use std::path::PathBuf;
use url::form_urlencoded;

/// JVM and Maven options are removed from the URL before it is passed to JDBC.
#[derive(Clone, Debug)]
pub struct Options {
    pub url: String,
    pub driver: Option<String>,
    pub classpath: Vec<PathBuf>,
    pub dependencies: Vec<ArtifactCoordinate>,
}

impl Options {
    /// Parse a JDBC URL, preserving database-specific parameters verbatim.
    ///
    /// # Errors
    /// Returns an error for an invalid JDBC URL, Maven coordinate, or JVM option.
    pub fn parse(url: &str) -> Result<Self> {
        let (base, query) = url.split_once('?').unwrap_or((url, ""));
        let Some((protocol, _)) = base.strip_prefix("jdbc:").and_then(|s| s.split_once(':')) else {
            return Err(Error::InvalidUrl(
                "Expected jdbc:<subprotocol>:<database>".into(),
            ));
        };
        if protocol.is_empty() {
            return Err(Error::InvalidUrl("Missing JDBC subprotocol".into()));
        }
        let mut options = Self {
            url: base.into(),
            driver: None,
            classpath: Vec::new(),
            dependencies: Vec::new(),
        };
        let mut database_parameters = Vec::new();
        for parameter in query.split('&').filter(|s| !s.is_empty()) {
            let Some((key, value)) = form_urlencoded::parse(parameter.as_bytes()).next() else {
                continue;
            };
            match key.as_ref() {
                "driver" | "driver_class" => {
                    if value.is_empty() || options.driver.is_some() {
                        return Err(Error::InvalidUrl(
                            "Specify one nonempty JDBC driver class".into(),
                        ));
                    }
                    options.driver = Some(value.into_owned());
                }
                "dependency" => {
                    options.dependencies.push(value.parse().map_err(|error| {
                        Error::InvalidUrl(format!("Invalid JDBC Maven dependency: {error}"))
                    })?);
                }
                "classpath" => {
                    if value.is_empty() {
                        return Err(Error::InvalidUrl("JDBC classpath cannot be empty".into()));
                    }
                    options
                        .classpath
                        .extend(std::env::split_paths(value.as_ref()));
                }
                _ => database_parameters.push(parameter),
            }
        }
        if !database_parameters.is_empty() {
            options.url.push('?');
            options.url.push_str(&database_parameters.join("&"));
        }
        if options.classpath.is_empty()
            && let Some(classpath) = std::env::var_os("CLASSPATH")
        {
            options.classpath.extend(std::env::split_paths(&classpath));
        }
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_jdbc_parameters() -> Result<()> {
        let options = Options::parse(
            "jdbc:postgresql://localhost/db?ssl=true&driver=org.postgresql.Driver&user=a%2Bb&classpath=%2Ftmp%2Fmy+driver.jar",
        )?;
        assert_eq!(
            options.url,
            "jdbc:postgresql://localhost/db?ssl=true&user=a%2Bb"
        );
        assert_eq!(options.driver.as_deref(), Some("org.postgresql.Driver"));
        assert_eq!(options.classpath, [PathBuf::from("/tmp/my driver.jar")]);
        Ok(())
    }

    #[test]
    fn optional_driver_and_h2_settings() -> Result<()> {
        let options = Options::parse("jdbc:h2:mem:test;DB_CLOSE_DELAY=-1")?;
        assert!(options.driver.is_none());
        assert_eq!(options.url, "jdbc:h2:mem:test;DB_CLOSE_DELAY=-1");
        Ok(())
    }

    #[test]
    fn dependencies_and_classpath_preserve_database_options() -> Result<()> {
        let options = Options::parse(
            "jdbc:postgresql://localhost/db?dependency=org.postgresql%3Apostgresql%3A42.7.13&user=a%2Bb&dependency=example:extension:jar:all:1.0&classpath=local.jar&ssl=true",
        )?;
        assert_eq!(
            options.url,
            "jdbc:postgresql://localhost/db?user=a%2Bb&ssl=true"
        );
        assert_eq!(options.classpath, [PathBuf::from("local.jar")]);
        assert_eq!(
            options
                .dependencies
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            [
                "org.postgresql:postgresql:42.7.13",
                "example:extension:jar:all:1.0"
            ]
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_options() {
        for url in [
            "h2:mem:",
            "jdbc:",
            "jdbc::db",
            "jdbc:h2:mem:?driver=",
            "jdbc:h2:mem:?driver=A&driver_class=B",
            "jdbc:h2:mem:?classpath=",
            "jdbc:h2:mem:?dependency=",
            "jdbc:h2:mem:?dependency=group:artifact",
            "jdbc:h2:mem:?dependency=group:artifact:",
            "jdbc:h2:mem:?dependency=group:artifact:..%2Fescape",
        ] {
            assert!(Options::parse(url).is_err(), "{url}");
        }
    }
}
