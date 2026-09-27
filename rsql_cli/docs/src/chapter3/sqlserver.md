# SQL Server

The `sqlserver` driver connects to Microsoft SQL Server using Tiberius.

## URL format

```text
sqlserver://[<user>[:<password>]@]<host>[:<port>][/<database>][?<options>]
```

## Options

| Option                            | Behavior / default                                                                                                               |
|-----------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| `encrypt`                         | `true`/`yes` requires encryption (default). `false`/`no` uses Tiberius Off mode; `DANGER_PLAINTEXT` disables encryption support. |
| `TrustServerCertificate`          | Skip certificate validation when `true`/`yes`; default `false`. Cannot be combined with `TrustServerCertificateCA`.              |
| `TrustServerCertificateCA`        | Certificate-authority file for validating the server.                                                                            |
| `IntegratedSecurity`              | Use Windows integrated authentication when `true`/`yes`; default `false`. Available only on Windows.                             |
| `ApplicationName`                 | Application name reported to the server; alias `Application Name`.                                                               |
| `server`                          | Override the URL host, optionally as `tcp:host,port`.                                                                            |
| `database`                        | Override the URL database.                                                                                                       |
| `uid / username / user / user id` | Override the URL username.                                                                                                       |
| `password / pwd`                  | Override the URL password.                                                                                                       |
| `encryption`                      | Legacy setting: `off`, `on`, `not_supported`, or `required`. Used only when `encrypt` is absent.                                 |

## Examples

```shell
rsql --url 'sqlserver://user:password@db.example.com:1433/example?encrypt=true&ApplicationName=rsql' -- 'SELECT @@VERSION;'
rsql --url 'sqlserver://localhost/example?IntegratedSecurity=true'
```

## Usage notes

Options are case-insensitive. The default host is `localhost` and the default port is `1433`. The
server chooses the database if omitted. Boolean options accept `true`, `false`, `yes`, and `no`. Use
`.schemas`, `.tables`, and `.describe` to inspect accessible objects.
