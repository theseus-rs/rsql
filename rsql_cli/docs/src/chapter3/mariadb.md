# MariaDB

The `mariadb` driver connects to MariaDB using SQLx.

## URL format

```text
mariadb://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
```

## Options

| Option                     | Behavior / default                                                                                                      |
|----------------------------|-------------------------------------------------------------------------------------------------------------------------|
| `ssl-mode`                 | TLS mode: `DISABLED`, `PREFERRED`, `REQUIRED`, `VERIFY_CA`, or `VERIFY_IDENTITY`. Default `PREFERRED`; alias `sslmode`. |
| `ssl-ca`                   | CA certificate file; alias `sslca`.                                                                                     |
| `ssl-cert / ssl-key`       | Client certificate and key files; aliases `sslcert` and `sslkey`.                                                       |
| `charset`                  | Connection character set; default `utf8mb4`.                                                                            |
| `collation`                | Connection collation; determined from the charset when omitted.                                                         |
| `timezone`                 | Session time zone; alias `time-zone`. Encode a plus sign as `%2B`.                                                      |
| `socket`                   | Local Unix socket path instead of TCP.                                                                                  |
| `statement-cache-capacity` | Prepared-statement cache size; default `100`.                                                                           |

## Examples

```shell
rsql --url 'mariadb://user:password@localhost:3306/example' -- 'SELECT VERSION();'
rsql --url 'mariadb://user:password@db.example.com/example?ssl-mode=VERIFY_IDENTITY&ssl-ca=/path/ca.pem'
```

## Usage notes

The default TCP port is `3306`. Percent-encode reserved characters in credentials and option values.
Use `.tables` and `.describe` to inspect the selected database. MariaDB and MySQL share the same
connection implementation.
