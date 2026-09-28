# CockroachDB

The `cockroachdb` driver connects to CockroachDB using the PostgreSQL protocol.

## URL format

```text
cockroachdb://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
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
rsql --url 'cockroachdb://user:password@localhost:26257/example' -- 'SELECT 1;'
```

## Usage notes

The connection uses the PostgreSQL wire protocol through SQLx. Specify the service port explicitly;
the underlying connection defaults to port `5432`. The example uses port `26257`. Use `.catalogs`,
`.schemas`, `.tables`, and `.describe` to inspect accessible objects.

See [PostgreSQL (SQLx)](postgresql.md) for shared connection options. SQL features and metadata
depend on the target service.
