# PostgreSQL (rust-postgres)

The `postgres` driver connects to PostgreSQL using the native rust-postgres client.

## URL format

```text
postgres://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
```

## Options

| Option                   | Behavior / default                                                                                                                 |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------|
| `application_name`       | Name reported for the session.                                                                                                     |
| `connect_timeout`        | Connection timeout in seconds.                                                                                                     |
| `options`                | URL-encoded server startup options.                                                                                                |
| `sslmode`                | This driver uses `NoTls`; `disable` explicitly selects an unencrypted connection. Use the `postgresql` driver for TLS connections. |
| `embedded`               | Start a managed local PostgreSQL server when `true`; default `false`.                                                              |
| `version`                | Embedded PostgreSQL version requirement; default `=18.6.0`.                                                                        |
| `installation_dir`       | Directory for the downloaded PostgreSQL installation.                                                                              |
| `data_dir`               | Embedded server data directory.                                                                                                    |
| `temporary`              | Remove temporary server data after use; defaults to `true`.                                                                        |
| `timeout`                | Embedded setup/start/stop command timeout in seconds.                                                                              |
| `releases_url`           | Download source for embedded PostgreSQL releases.                                                                                  |
| `password_file`          | Path to the embedded server password file.                                                                                         |
| `socket_dir`             | Unix socket directory for the embedded server.                                                                                     |
| `trust_installation_dir` | Trust an existing installation directory when `true`; default `false`.                                                             |
| `configuration.<name>`   | Embedded PostgreSQL server setting, for example `configuration.max_connections=100`.                                               |

## Examples

```shell
rsql --url 'postgres://postgres@localhost:5432/example?sslmode=disable' -- 'SELECT version();'
rsql --url 'postgres://?embedded=true' -- 'SELECT 42;'
```

## Usage notes

The default PostgreSQL port is `5432`. Connection parameters are parsed by rust-postgres.

With `embedded=true`, rsql downloads and starts a managed PostgreSQL instance and creates the
`embedded` database. It stops the server when the connection closes. The first setup needs network
access unless the PostgreSQL installation is already cached. For TLS connections, use [PostgreSQL
(SQLx)](postgresql.md).
