# Amazon Redshift

The `redshift` driver connects to Amazon Redshift using the PostgreSQL protocol.

## URL format

```text
redshift://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
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

## Examples

```shell
rsql --url 'redshift://user:password@cluster.example.com:5439/example?sslmode=verify-full' -- 'SELECT 1;'
```

## Usage notes

The connection uses the PostgreSQL wire protocol through SQLx. Specify the service port explicitly;
the underlying connection defaults to port `5432`. The example uses port `5439`. Use `.catalogs`,
`.schemas`, `.tables`, and `.describe` to inspect accessible objects.

See [PostgreSQL (SQLx)](postgresql.md) for shared connection options. SQL features and metadata
depend on the target service.
