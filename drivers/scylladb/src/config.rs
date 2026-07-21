use percent_encoding::percent_decode_str;
use rsql_driver::Error::InvalidUrl;
use rsql_driver::Result;
use std::path::PathBuf;
use url::{Host, Url};

const DEFAULT_PORT: u16 = 9042;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SslMode {
    Disable,
    VerifyFull,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Config {
    pub(crate) nodes: Vec<String>,
    pub(crate) keyspace: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) datacenter: Option<String>,
    pub(crate) ssl_mode: SslMode,
    pub(crate) ssl_ca: Option<PathBuf>,
    pub(crate) ssl_cert: Option<PathBuf>,
    pub(crate) ssl_key: Option<PathBuf>,
    pub(crate) client_routes: Vec<String>,
}

impl Config {
    #[expect(
        clippy::too_many_lines,
        reason = "URL parsing validates the complete public option surface together"
    )]
    pub(crate) fn from_url(url: &str) -> Result<Self> {
        let parsed = Url::parse(url).map_err(|error| InvalidUrl(error.to_string()))?;
        if parsed.scheme() != "scylladb" {
            return Err(InvalidUrl(format!(
                "expected scylladb URL scheme, found {}",
                parsed.scheme()
            )));
        }

        let host = parsed
            .host()
            .ok_or_else(|| InvalidUrl("ScyllaDB URL requires a host".to_string()))?;
        let primary_node = format_host(&host, parsed.port().unwrap_or(DEFAULT_PORT));

        let username = (!parsed.username().is_empty())
            .then(|| decode_component(parsed.username(), "username"))
            .transpose()?;
        let password = parsed
            .password()
            .map(|password| decode_component(password, "password"))
            .transpose()?;
        if username.is_some() != password.is_some() {
            return Err(InvalidUrl(
                "ScyllaDB username and password must be provided together".to_string(),
            ));
        }

        let path = parsed.path().trim_matches('/');
        if path.contains('/') {
            return Err(InvalidUrl(
                "ScyllaDB URL path may contain only one keyspace".to_string(),
            ));
        }
        let keyspace = (!path.is_empty())
            .then(|| decode_component(path, "keyspace"))
            .transpose()?;

        let mut nodes = vec![primary_node];
        let mut datacenter = None;
        let mut ssl_mode = SslMode::Disable;
        let mut ssl_ca = None;
        let mut ssl_cert = None;
        let mut ssl_key = None;
        let mut client_routes = Vec::new();

        for (name, value) in parsed.query_pairs() {
            let value = value.into_owned();
            match name.as_ref() {
                "node" => {
                    if value.is_empty() {
                        return Err(InvalidUrl("node must not be empty".to_string()));
                    }
                    nodes.push(with_default_port(&value));
                }
                "datacenter" => set_once(&mut datacenter, value, "datacenter")?,
                "sslmode" => {
                    ssl_mode = match value.as_str() {
                        "disable" => SslMode::Disable,
                        "verify-full" => SslMode::VerifyFull,
                        _ => {
                            return Err(InvalidUrl(format!(
                                "unsupported sslmode {value}; expected disable or verify-full"
                            )));
                        }
                    };
                }
                "ssl_ca" => set_path_once(&mut ssl_ca, value, "ssl_ca")?,
                "ssl_cert" => set_path_once(&mut ssl_cert, value, "ssl_cert")?,
                "ssl_key" => set_path_once(&mut ssl_key, value, "ssl_key")?,
                "client_route" => {
                    if value.is_empty() {
                        return Err(InvalidUrl("client_route must not be empty".to_string()));
                    }
                    client_routes.push(value);
                }
                unknown => {
                    return Err(InvalidUrl(format!(
                        "unsupported ScyllaDB URL option: {unknown}"
                    )));
                }
            }
        }

        if ssl_cert.is_some() != ssl_key.is_some() {
            return Err(InvalidUrl(
                "ssl_cert and ssl_key must be provided together".to_string(),
            ));
        }
        let has_tls_files = ssl_ca.is_some() || ssl_cert.is_some() || ssl_key.is_some();
        if ssl_mode == SslMode::Disable && has_tls_files {
            return Err(InvalidUrl(
                "TLS certificate options require sslmode=verify-full".to_string(),
            ));
        }
        if !client_routes.is_empty() && (ssl_mode != SslMode::Disable || has_tls_files) {
            return Err(InvalidUrl(
                "Client Routes does not currently support TLS options".to_string(),
            ));
        }

        Ok(Self {
            nodes,
            keyspace,
            username,
            password,
            datacenter,
            ssl_mode,
            ssl_ca,
            ssl_cert,
            ssl_key,
            client_routes,
        })
    }
}

fn format_host(host: &Host<&str>, port: u16) -> String {
    match host {
        Host::Ipv6(address) => format!("[{address}]:{port}"),
        Host::Ipv4(address) => format!("{address}:{port}"),
        Host::Domain(domain) => format!("{domain}:{port}"),
    }
}

fn with_default_port(node: &str) -> String {
    if node.starts_with('[') || node.rsplit_once(':').is_some() {
        node.to_string()
    } else {
        format!("{node}:{DEFAULT_PORT}")
    }
}

fn set_once(target: &mut Option<String>, value: String, name: &str) -> Result<()> {
    if target.replace(value).is_some() {
        return Err(InvalidUrl(format!("{name} may only be specified once")));
    }
    Ok(())
}

fn set_path_once(target: &mut Option<PathBuf>, value: String, name: &str) -> Result<()> {
    if value.is_empty() {
        return Err(InvalidUrl(format!("{name} must not be empty")));
    }
    if target.replace(PathBuf::from(value)).is_some() {
        return Err(InvalidUrl(format!("{name} may only be specified once")));
    }
    Ok(())
}

fn decode_component(value: &str, name: &str) -> Result<String> {
    percent_decode_str(value)
        .decode_utf8()
        .map(String::from)
        .map_err(|error| InvalidUrl(format!("invalid UTF-8 in ScyllaDB {name}: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_direct_url() -> Result<()> {
        let config = Config::from_url(
            "scylladb://user:secret@db.example/analytics?node=db2:9142&node=db3&datacenter=dc1",
        )?;
        assert_eq!(config.nodes, ["db.example:9042", "db2:9142", "db3:9042"]);
        assert_eq!(config.keyspace.as_deref(), Some("analytics"));
        assert_eq!(config.username.as_deref(), Some("user"));
        assert_eq!(config.password.as_deref(), Some("secret"));
        assert_eq!(config.datacenter.as_deref(), Some("dc1"));
        Ok(())
    }

    #[test]
    fn parses_tls_and_routes() -> Result<()> {
        let tls = Config::from_url(
            "scylladb://cloud.example?sslmode=verify-full&ssl_ca=ca.pem&ssl_cert=client.pem&ssl_key=client.key",
        )?;
        assert_eq!(tls.ssl_mode, SslMode::VerifyFull);
        assert_eq!(tls.ssl_ca, Some(PathBuf::from("ca.pem")));

        let routes = Config::from_url(
            "scylladb://private.example?client_route=route-a&client_route=route-b",
        )?;
        assert_eq!(routes.client_routes, ["route-a", "route-b"]);
        Ok(())
    }

    #[test]
    fn decodes_url_components() -> Result<()> {
        let config = Config::from_url("scylladb://user%40name:p%40ss@localhost/My%20Keyspace")?;
        assert_eq!(config.username.as_deref(), Some("user@name"));
        assert_eq!(config.password.as_deref(), Some("p@ss"));
        assert_eq!(config.keyspace.as_deref(), Some("My Keyspace"));
        Ok(())
    }

    #[test]
    fn rejects_invalid_options() {
        for url in [
            "scylladb://user@localhost",
            "scylladb://localhost?ssl_cert=client.pem",
            "scylladb://localhost?sslmode=verify-full&client_route=route-a",
            "scylladb://localhost?unknown=true",
            "scylladb://localhost/one/two",
        ] {
            assert!(Config::from_url(url).is_err(), "URL should fail: {url}");
        }
    }
}
