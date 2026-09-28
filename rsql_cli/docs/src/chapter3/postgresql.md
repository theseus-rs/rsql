# PostgreSQL (SQLx)

The `postgresql` driver connects to PostgreSQL (SQLx) using the PostgreSQL protocol.

## URL format

```text
postgresql://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
```

## Options

| Option                                              | Behavior / default                                                                                                                                |
|-----------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| `sslmode`                                           | TLS mode: `disable`, `allow`, `prefer`, `require`, `verify-ca`, or `verify-full`. SQLx defaults to `prefer` unless configured by the environment. |
| `sslrootcert`                                       | CA certificate file. Aliases: `ssl-root-cert`, `ssl-ca`.                                                                                          |
| `sslcert / sslkey`                                  | Client certificate and key files for certificate authentication.                                                                                  |
| `application_name`                                  | Name reported for the PostgreSQL session.                                                                                                         |
| `options`                                           | Server startup options, for example `-c search_path=public`, URL-encoded.                                                                         |
| `statement-cache-capacity`                          | Prepared-statement cache size per connection; default `100`.                                                                                      |
| `host / hostaddr / port / dbname / user / password` | Connection fields; query values override matching URL fields.                                                                                     |
| `embedded`                                          | Start a managed local PostgreSQL server when `true`; default `false`.                                                                             |
| `version`                                           | Embedded PostgreSQL version requirement; default `=18.6.0`.                                                                                       |
| `installation_dir`                                  | Directory for the downloaded PostgreSQL installation.                                                                                             |
| `data_dir`                                          | Embedded server data directory.                                                                                                                   |
| `temporary`                                         | Remove temporary server data after use; defaults to `true`.                                                                                       |
| `timeout`                                           | Embedded setup/start/stop command timeout in seconds.                                                                                             |
| `releases_url`                                      | Download source for embedded PostgreSQL releases.                                                                                                 |
| `password_file`                                     | Path to the embedded server password file.                                                                                                        |
| `socket_dir`                                        | Unix socket directory for the embedded server.                                                                                                    |
| `trust_installation_dir`                            | Trust an existing installation directory when `true`; default `false`.                                                                            |
| `configuration.<name>`                              | Embedded PostgreSQL server setting, for example `configuration.max_connections=100`.                                                              |

## Examples

```shell
rsql --url 'postgresql://user:password@localhost:5432/example' -- 'SELECT 1;'
rsql --url 'postgresql://?embedded=true' -- 'SELECT version();'
```

## Usage notes

The connection uses the PostgreSQL wire protocol through SQLx. Specify the service port explicitly;
the underlying connection defaults to port `5432`. The example uses port `5432`. Use `.catalogs`,
`.schemas`, `.tables`, and `.describe` to inspect accessible objects.

With `embedded=true`, rsql downloads and starts PostgreSQL, creates a database named `embedded`, and
stops the server when the connection closes. Embedded options apply only in this mode. The initial
setup needs network access unless the installation is already cached.
