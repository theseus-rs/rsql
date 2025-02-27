## drivers

### Usage

```text
.drivers
```

### Description

The drivers command displays the available data drivers.

| Driver        | Description                                                                                            | URL                                                                                                                       |
|---------------|--------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------|
| `arrow`       | Arrow IPC provided by [Polars](https://github.com/pola-rs/polars)                                      | `arrow://<file>`                                                                                                          |
| `avro`        | Avro provided by [Polars](https://github.com/pola-rs/polars)                                           | `avro://<file>`                                                                                                           |
| `cockroachdb` | CockroachDB provided by [SQLx](https://github.com/launchbadge/sqlx)                                    | `cockroachdb://<user>[:<password>]@<host>[:<port>]/<database>`                                                            |
| `csv`         | Comma Separated Value (CSV) provided by [Polars](https://github.com/pola-rs/polars)                    | `csv://<file>[?has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                                                  |
| `delimited`   | Delimited provided by [Polars](https://github.com/pola-rs/polars)                                      | `delimited://<file>[?separator=<char>][&has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                         |
| `duckdb`      | DuckDB provided by [DuckDB](https://duckdb.org/)                                                       | `duckdb://[<file>]`                                                                                                       |
| `excel`       | Excel                                                                                                  | `excel://<file>[?has_header=<true\|false>][&skip_rows=<n>]`                                                               |
| `file`        | File                                                                                                   | `file://<file>`                                                                                                           |
| `http`        | HTTP                                                                                                   | `http://<path>[?_headers=<headers>]`                                                                                      |
| `https`       | HTTPS                                                                                                  | `https://<path>[?_headers=<headers>]`                                                                                     |
| `json`        | JSON provided by [Polars](https://github.com/pola-rs/polars)                                           | `json://<file>`                                                                                                           |
| `jsonl`       | JSONL provided by [Polars](https://github.com/pola-rs/polars)                                          | `jsonl://<file>`                                                                                                          |
| `libsql`      | LibSQL provided by [Turso](https://github.com/tursodatabase/libsql)                                    | `libsql://<host>?[<memory=true>][&file=<database_file>][&auth_token=<token>]`                                             |
| `mariadb`     | MariaDB provided by [SQLx](https://github.com/launchbadge/sqlx)                                        | `mariadb://<user>[:<password>]@<host>[:<port>]/<database>`                                                                |
| `mysql`       | MySQL provided by [SQLx](https://github.com/launchbadge/sqlx)                                          | `mysql://<user>[:<password>]@<host>[:<port>]/<database>`                                                                  |
| `ods`         | OpenDocument Spreadsheet                                                                               | `ods://<file>[?has_header=<true\|false>][&skip_rows=<n>]`                                                                 |
| `parquet`     | Parquet provided by [Polars](https://github.com/pola-rs/polars)                                        | `parquet://<file>`                                                                                                        |
| `postgres`    | PostgreSQL provided by [rust-postgres](https://github.com/sfackler/rust-postgres)                      | `postgres://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                               |
| `postgresql`  | PostgreSQL provided by [SQLx](https://github.com/launchbadge/sqlx)                                     | `postgresql://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                             |
| `redshift`    | Redshift provided by [SQLx](https://github.com/launchbadge/sqlx)                                       | `redshift://<user>[:<password>]@<host>[:<port>]/<database>`                                                               |
| `rusqlite`    | SQLite provided by [Rusqlite](https://github.com/rusqlite/rusqlite?tab=readme-ov-file#rusqlite)        | `rusqlite://[<file>]`                                                                                                     |
| `snowflake`   | Snowflake provided by [Snowflake SQL API](https://docs.snowflake.com/en/developer-guide/sql-api/index) | `snowflake://<user>[:<token>]@<account>.snowflakecomputing.com/[?private_key_file=pkey_file&public_key_file=pubkey_file]` |
| `sqlite`      | SQLite provided by [SQLx](https://github.com/launchbadge/sqlx)                                         | `sqlite://[<file>]`                                                                                                       |
| `sqlserver`   | SQL Server provided by [Tiberius](https://github.com/prisma/tiberius)                                  | `sqlserver://<user>[:<password>]@<host>[:<port>]/<database>`                                                              |
| `tsv`         | Tab Separated Value (TSV) provided by [Polars](https://github.com/pola-rs/polars)                      | `tsv://<file>[?has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                                                  |
| `xml`         | Extensible Markup Language (XML) provided by [Polars](https://github.com/pola-rs/polars)               | `xml://<file>`                                                                                                            |
| `yaml`        | Extensible Markup Language (YAML) provided by [Polars](https://github.com/pola-rs/polars)              | `yaml://<file>`                                                                                                           |

### Examples

Show the available drivers:

```text
.drivers
```

### Demonstration

![](./demo.gif)
