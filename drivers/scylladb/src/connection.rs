use crate::config::{Config, SslMode};
use crate::convert::{cql_to_value, values_to_cql};
use async_trait::async_trait;
use file_type::FileType;
use futures_util::TryStreamExt;
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{MemoryQueryResult, Metadata, QueryResult, Result, StatementMetadata, ToSql};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ClientConfig, RootCertStore};
use scylla::client::caching_session::CachingSession;
use scylla::client::client_routes::{ClientRoutesConfig, ClientRoutesProxy};
use scylla::client::pager::QueryPager;
use scylla::client::session::Session;
use scylla::client::session_builder::{ClientRoutesSessionBuilder, SessionBuilder};
use scylla::statement::prepared::PreparedStatement;
use scylla::statement::unprepared::Statement;
use scylla::value::Row;
use std::path::Path;
use std::sync::Arc;

const PREPARED_CACHE_CAPACITY: usize = 128;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "scylladb"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        Ok(Box::new(Box::pin(Connection::new(url)).await?))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

pub struct Connection {
    url: String,
    pub(crate) session: CachingSession,
}

impl Connection {
    /// Connect to `ScyllaDB` using an rsql `scylladb://` URL.
    ///
    /// # Errors
    /// Returns an error when the URL is invalid, TLS material cannot be loaded, or the cluster
    /// cannot be reached.
    pub async fn new(url: &str) -> Result<Self> {
        let config = Config::from_url(url)?;
        let session = Box::pin(build_session(&config)).await?;
        Ok(Self {
            url: url.to_string(),
            session: CachingSession::from(session, PREPARED_CACHE_CAPACITY),
        })
    }

    async fn prepare(
        &self,
        sql: &str,
        params: &[&dyn ToSql],
    ) -> Result<(PreparedStatement, Vec<Option<scylla::value::CqlValue>>)> {
        let statement = Statement::new(self.keyspace_cache_key(sql));
        let prepared = self
            .session
            .add_prepared_statement(&statement)
            .await
            .map_err(|error| IoError(error.to_string()))?;
        let values = rsql_driver::to_values(params);
        let specs = prepared.get_variable_col_specs();
        let column_types = specs
            .iter()
            .map(scylla::frame::response::result::ColumnSpec::typ)
            .collect::<Vec<_>>();
        let values = values_to_cql(&values, &column_types)?;
        Ok((prepared, values))
    }

    fn keyspace_cache_key(&self, sql: &str) -> String {
        self.session.get_session().get_keyspace().map_or_else(
            || sql.to_string(),
            |keyspace| format!("/* rsql:keyspace={keyspace} */ {sql}"),
        )
    }

    async fn collect_query(pager: QueryPager) -> Result<MemoryQueryResult> {
        let columns = pager
            .column_specs()
            .iter()
            .map(|column| column.name().to_string())
            .collect();
        let mut stream = pager
            .rows_stream::<Row>()
            .map_err(|error| IoError(error.to_string()))?;
        let mut rows = Vec::new();
        while let Some(row) = stream
            .try_next()
            .await
            .map_err(|error| IoError(error.to_string()))?
        {
            rows.push(
                row.columns
                    .into_iter()
                    .map(|value| value.map_or(rsql_driver::Value::Null, cql_to_value))
                    .collect(),
            );
        }
        Ok(MemoryQueryResult::new(columns, rows))
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        if let Some((keyspace, case_sensitive)) = parse_use_keyspace(sql) {
            if !params.is_empty() {
                return Err(IoError("USE does not accept bound parameters".to_string()));
            }
            self.session
                .get_session()
                .use_keyspace(keyspace, case_sensitive)
                .await
                .map_err(|error| IoError(error.to_string()))?;
            return Ok(0);
        }

        if params.is_empty() {
            self.session
                .get_session()
                .query_unpaged(sql, ())
                .await
                .map_err(|error| IoError(error.to_string()))?;
        } else {
            let (prepared, values) = self.prepare(sql, params).await?;
            self.session
                .get_session()
                .execute_unpaged(&prepared, &values)
                .await
                .map_err(|error| IoError(error.to_string()))?;
        }
        Ok(0)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let pager = if params.is_empty() {
            self.session
                .get_session()
                .query_iter(sql, ())
                .await
                .map_err(|error| IoError(error.to_string()))?
        } else {
            let (prepared, values) = self.prepare(sql, params).await?;
            self.session
                .get_session()
                .execute_iter(prepared, &values)
                .await
                .map_err(|error| IoError(error.to_string()))?
        };
        Ok(Box::new(Self::collect_query(pager).await?))
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        crate::metadata::get_metadata(self).await
    }

    fn parse_sql(&self, sql: &str) -> StatementMetadata {
        classify_cql(sql)
    }
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Connection")
            .field("url", &self.url)
            .field("session", &"<ScyllaDB session>")
            .finish()
    }
}

async fn build_session(config: &Config) -> Result<Session> {
    if config.client_routes.is_empty() {
        let mut builder = SessionBuilder::new().known_nodes(&config.nodes);
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            builder = builder.user(username, password);
        }
        if let Some(datacenter) = &config.datacenter {
            builder = builder.prefer_datacenter(datacenter.clone());
        }
        if let Some(keyspace) = &config.keyspace {
            builder = builder.use_keyspace(keyspace, false);
        }
        if config.ssl_mode == SslMode::VerifyFull {
            builder = builder.tls_context(Some(build_tls_config(config)?));
        }
        Box::pin(builder.build())
            .await
            .map_err(|error| IoError(error.to_string()))
    } else {
        let proxies = config
            .client_routes
            .iter()
            .cloned()
            .map(ClientRoutesProxy::new_with_connection_id)
            .collect();
        let routes =
            ClientRoutesConfig::new(proxies).map_err(|error| InvalidUrl(error.to_string()))?;
        let mut builder = ClientRoutesSessionBuilder::new(routes).known_nodes(&config.nodes);
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            builder = builder.user(username, password);
        }
        if let Some(datacenter) = &config.datacenter {
            builder = builder.prefer_datacenter(datacenter.clone());
        }
        if let Some(keyspace) = &config.keyspace {
            builder = builder.use_keyspace(keyspace, false);
        }
        Box::pin(builder.build())
            .await
            .map_err(|error| IoError(error.to_string()))
    }
}

fn build_tls_config(config: &Config) -> Result<Arc<ClientConfig>> {
    let mut roots = RootCertStore::empty();
    let certificates = if let Some(ca_file) = &config.ssl_ca {
        load_certificates(ca_file)?
    } else {
        let native = rustls_native_certs::load_native_certs();
        if native.certs.is_empty() {
            return Err(IoError(format!(
                "no native root certificates could be loaded: {:?}",
                native.errors
            )));
        }
        native.certs
    };
    let (valid, _) = roots.add_parsable_certificates(certificates);
    if valid == 0 {
        return Err(IoError(
            "no valid root certificates were loaded".to_string(),
        ));
    }

    let provider = Arc::new(crypto_provider());
    let builder = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|error| IoError(error.to_string()))?
        .with_root_certificates(roots);
    let client_config =
        if let (Some(cert_file), Some(key_file)) = (&config.ssl_cert, &config.ssl_key) {
            let certificates = load_certificates(cert_file)?;
            if certificates.is_empty() {
                return Err(IoError(format!(
                    "no client certificates found in {}",
                    cert_file.display()
                )));
            }
            let key = PrivateKeyDer::from_pem_file(key_file).map_err(|error| {
                IoError(format!(
                    "failed to load private key from {}: {error}",
                    key_file.display()
                ))
            })?;
            builder
                .with_client_auth_cert(certificates, key)
                .map_err(|error| IoError(error.to_string()))?
        } else {
            builder.with_no_client_auth()
        };
    Ok(Arc::new(client_config))
}

fn load_certificates(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    CertificateDer::pem_file_iter(path)
        .map_err(|error| {
            IoError(format!(
                "failed to load certificates from {}: {error}",
                path.display()
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| IoError(error.to_string()))
}

fn crypto_provider() -> rustls::crypto::CryptoProvider {
    #[cfg(any(feature = "tls-native-tls", feature = "tls-rustls-aws-lc-rs"))]
    {
        rustls::crypto::aws_lc_rs::default_provider()
    }
    #[cfg(all(
        not(any(feature = "tls-native-tls", feature = "tls-rustls-aws-lc-rs")),
        feature = "tls-rustls-ring"
    ))]
    {
        rustls::crypto::ring::default_provider()
    }
    #[cfg(not(any(
        feature = "tls-native-tls",
        feature = "tls-rustls-aws-lc-rs",
        feature = "tls-rustls-ring"
    )))]
    {
        rustls::crypto::aws_lc_rs::default_provider()
    }
}

pub(crate) fn classify_cql(sql: &str) -> StatementMetadata {
    match first_keyword(sql).as_deref() {
        Some("SELECT") => StatementMetadata::Query,
        Some("INSERT" | "UPDATE" | "DELETE" | "BEGIN" | "TRUNCATE") => StatementMetadata::DML,
        Some("CREATE" | "ALTER" | "DROP" | "USE") => StatementMetadata::DDL,
        _ => StatementMetadata::Unknown,
    }
}

fn first_keyword(sql: &str) -> Option<String> {
    let remaining = strip_leading_comments(sql)?;
    let end = remaining
        .find(|character: char| !character.is_ascii_alphabetic())
        .unwrap_or(remaining.len());
    remaining
        .get(..end)
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_uppercase)
}

fn strip_leading_comments(sql: &str) -> Option<&str> {
    let mut remaining = sql.trim_start();
    loop {
        if remaining.starts_with("--") || remaining.starts_with("//") {
            remaining = remaining.split_once('\n')?.1.trim_start();
        } else if let Some(comment) = remaining.strip_prefix("/*") {
            remaining = comment.split_once("*/")?.1.trim_start();
        } else {
            break;
        }
    }
    Some(remaining)
}

fn parse_use_keyspace(sql: &str) -> Option<(String, bool)> {
    let remaining = strip_leading_comments(sql)?;
    let identifier = remaining
        .get(3..)
        .filter(|_| first_keyword(remaining).as_deref() == Some("USE"))?
        .trim()
        .trim_end_matches(';')
        .trim();
    if let Some(quoted) = identifier
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        Some((quoted.replace("\"\"", "\""), true))
    } else if identifier.is_empty() {
        None
    } else {
        Some((identifier.to_string(), false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_cql_statements() {
        assert!(matches!(
            classify_cql("SELECT * FROM t"),
            StatementMetadata::Query
        ));
        assert!(matches!(
            classify_cql("BEGIN BATCH"),
            StatementMetadata::DML
        ));
        assert!(matches!(classify_cql("TRUNCATE t"), StatementMetadata::DML));
        assert!(matches!(
            classify_cql("/* comment */ USE analytics"),
            StatementMetadata::DDL
        ));
        assert!(matches!(
            classify_cql("LIST USERS"),
            StatementMetadata::Unknown
        ));
    }

    #[test]
    fn parses_use_statements() {
        assert_eq!(
            parse_use_keyspace("USE analytics;"),
            Some(("analytics".to_string(), false))
        );
        assert_eq!(
            parse_use_keyspace("USE \"Analytics\""),
            Some(("Analytics".to_string(), true))
        );
        assert_eq!(
            parse_use_keyspace("/* USE ignored */ USE analytics"),
            Some(("analytics".to_string(), false))
        );
    }

    #[test]
    fn rejects_invalid_tls_files() -> Result<()> {
        let config =
            Config::from_url("scylladb://localhost?sslmode=verify-full&ssl_ca=missing.pem")?;
        assert!(build_tls_config(&config).is_err());
        Ok(())
    }

    #[test]
    fn loads_tls_and_mtls_pem_files() -> anyhow::Result<()> {
        let rcgen::CertifiedKey { cert, signing_key } =
            rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
        let directory = tempfile::tempdir()?;
        let certificate_file = directory.path().join("client.pem");
        let key_file = directory.path().join("client.key");
        std::fs::write(&certificate_file, cert.pem())?;
        std::fs::write(&key_file, signing_key.serialize_pem())?;

        let mut url = url::Url::parse("scylladb://localhost")?;
        url.query_pairs_mut()
            .append_pair("sslmode", "verify-full")
            .append_pair("ssl_ca", &certificate_file.to_string_lossy())
            .append_pair("ssl_cert", &certificate_file.to_string_lossy())
            .append_pair("ssl_key", &key_file.to_string_lossy());
        let config = Config::from_url(url.as_str())?;
        let _tls = build_tls_config(&config)?;
        Ok(())
    }
}
