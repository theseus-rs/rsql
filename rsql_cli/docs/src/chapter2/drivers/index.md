## drivers

### Usage

```text
.drivers
```

### Description

The drivers command displays the available data drivers.

| Driver        | Description                                                                                            | URL                                                                                                                                                                              |
|---------------|--------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `arrow`       | Arrow IPC provided by [Polars](https://github.com/pola-rs/polars)                                      | `arrow://<file>`                                                                                                                                                                 |
| `avro`        | Avro provided by [Polars](https://github.com/pola-rs/polars)                                           | `avro://<file>`                                                                                                                                                                  |
| `brotli`      | Brotli compressed file                                                                                 | `brotli://<file>`                                                                                                                                                                |
| `bzip2`       | Bzip2 compressed file                                                                                  | `bzip2://<file>`                                                                                                                                                                 |
| `cockroachdb` | CockroachDB provided by [SQLx](https://github.com/launchbadge/sqlx)                                    | `cockroachdb://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                   |
| `csv`         | Comma Separated Value (CSV) provided by [Polars](https://github.com/pola-rs/polars)                    | `csv://<file>[?has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                                                                                                         |
| `delimited`   | Delimited provided by [Polars](https://github.com/pola-rs/polars)                                      | `delimited://<file>[?separator=<char>][&has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                                                                                |
| `duckdb`      | DuckDB provided by [DuckDB](https://duckdb.org/)                                                       | `duckdb://[<file>]`                                                                                                                                                              |
| `dynamodb`    | DynamoDB                                                                                               | `dynamodb://[<access_key_id>:<secret_access_key>@]<host>[:<port>]>[?region=<region>][&session_token=<token>][&scheme=<http\|https>]`                                             |
| `excel`       | Excel                                                                                                  | `excel://<file>[?has_header=<true\|false>][&skip_rows=<n>]`                                                                                                                      |
| `file`        | File                                                                                                   | `file://<file>`                                                                                                                                                                  |
| `fwf`         | Fixed Width Format                                                                                     | `fwf://<file>?widths=<widths>[&headers=<headers>]`                                                                                                                               |
| `gzip`        | Gzip compressed file                                                                                   | `gzip://<file>`                                                                                                                                                                  |
| `http`        | HTTP                                                                                                   | `http://<path>[?_headers=<headers>]`                                                                                                                                             |
| `https`       | HTTPS                                                                                                  | `https://<path>[?_headers=<headers>]`                                                                                                                                            |
| `json`        | JSON provided by [Polars](https://github.com/pola-rs/polars)                                           | `json://<file>`                                                                                                                                                                  |
| `jsonl`       | JSONL provided by [Polars](https://github.com/pola-rs/polars)                                          | `jsonl://<file>`                                                                                                                                                                 |
| `libsql`      | LibSQL provided by [Turso](https://github.com/tursodatabase/libsql)                                    | `libsql://<host>?[<memory=true>][&file=<database_file>][&auth_token=<token>]`                                                                                                    |
| `lz4`         | LZ4 compressed file                                                                                    | `lz4://<file>`                                                                                                                                                                   |
| `mariadb`     | MariaDB provided by [SQLx](https://github.com/launchbadge/sqlx)                                        | `mariadb://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                       |
| `mysql`       | MySQL provided by [SQLx](https://github.com/launchbadge/sqlx)                                          | `mysql://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                         |
| `ods`         | OpenDocument Spreadsheet                                                                               | `ods://<file>[?has_header=<true\|false>][&skip_rows=<n>]`                                                                                                                        |
| `orc`         | Optimized Row Columnar (ORC)                                                                           | `orc://<file>`                                                                                                                                                                   |
| `parquet`     | Parquet provided by [Polars](https://github.com/pola-rs/polars)                                        | `parquet://<file>`                                                                                                                                                               |
| `postgres`    | PostgreSQL provided by [rust-postgres](https://github.com/sfackler/rust-postgres)                      | `postgres://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                                                                                      |
| `postgresql`  | PostgreSQL provided by [SQLx](https://github.com/launchbadge/sqlx)                                     | `postgresql://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                                                                                    |
| `redshift`    | Redshift provided by [SQLx](https://github.com/launchbadge/sqlx)                                       | `redshift://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                      |
| `rusqlite`    | SQLite provided by [Rusqlite](https://github.com/rusqlite/rusqlite?tab=readme-ov-file#rusqlite)        | `rusqlite://[<file>]`                                                                                                                                                            |
| `s3`          | Simple Storage Service (S3)                                                                            | `s3://[<access_key_id>:<secret_access_key>@]<host>[:<port>]/<bucket>/<object>[?region=<region>][&session_token=<token>][&force_path_style=(true\|false)][&scheme=<http\|https>]` |
| `snowflake`   | Snowflake provided by [Snowflake SQL API](https://docs.snowflake.com/en/developer-guide/sql-api/index) | `snowflake://<user>[:<token>]@<account>.snowflakecomputing.com/[?private_key_file=pkey_file&public_key_file=pubkey_file]`                                                        |
| `sqlite`      | SQLite provided by [SQLx](https://github.com/launchbadge/sqlx)                                         | `sqlite://[<file>]`                                                                                                                                                              |
| `sqlserver`   | SQL Server provided by [Tiberius](https://github.com/prisma/tiberius)                                  | `sqlserver://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                     |
| `tsv`         | Tab Separated Value (TSV) provided by [Polars](https://github.com/pola-rs/polars)                      | `tsv://<file>[?has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                                                                                                         |
| `xml`         | Extensible Markup Language (XML) provided by [Polars](https://github.com/pola-rs/polars)               | `xml://<file>`                                                                                                                                                                   |
| `xz`          | XZ compressed file                                                                                     | `xz://<file>`                                                                                                                                                                    |
| `yaml`        | Extensible Markup Language (YAML) provided by [Polars](https://github.com/pola-rs/polars)              | `yaml://<file>`                                                                                                                                                                  |
| `zstd`        | Zstd compressed file                                                                                   | `zstd://<file>`                                                                                                                                                                  |

### Examples

Show the available drivers:

```text
.drivers
```

### Demonstration

![](./demo.gif)
