## drivers

### Usage

```text
.drivers
```

### Description

The drivers command displays the available database drivers.

| Driver       | Description                                                                                            | URL                                                                                                                       |
|--------------|--------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------|
| `duckdb`     | DuckDB provided by [DuckDB](https://duckdb.org/)                                                       | `duckdb://?<memory=true>[&file=<database_file>]`                                                                          |
| `libsql`     | LibSQL provided by [Turso](https://github.com/tursodatabase/libsql)                                    | `libsql://<host>?[<memory=true>][&file=<database_file>][&auth_token=<token>]`                                             |
| `mariadb`    | MariaDB provided by [SQLx](https://github.com/launchbadge/sqlx)                                        | `mariadb://<user>[:<password>]@<host>[:<port>]/<database>`                                                                |
| `mysql`      | MySQL provided by [SQLx](https://github.com/launchbadge/sqlx)                                          | `mysql://<user>[:<password>]@<host>[:<port>]/<database>`                                                                  |
| `postgres`   | PostgreSQL driver provided by [rust-postgres](https://github.com/sfackler/rust-postgres)               | `postgres://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                               |
| `postgresql` | PostgreSQL driver provided by [SQLx](https://github.com/launchbadge/sqlx)                              | `postgresql://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                             |
| `redshift`   | Redshift driver provided by [SQLx](https://github.com/launchbadge/sqlx)                                | `redshift://<user>[:<password>]@<host>[:<port>]/<database>`                                                               |
| `rusqlite`   | SQLite provided by [Rusqlite](https://github.com/rusqlite/rusqlite?tab=readme-ov-file#rusqlite)        | `rusqlite://?<memory=true>[&file=<database_file>]`                                                                        |
| `snowflake`  | Snowflake provided by [Snowflake SQL API](https://docs.snowflake.com/en/developer-guide/sql-api/index) | `snowflake://<user>[:<token>]@<account>.snowflakecomputing.com/[?private_key_file=pkey_file&public_key_file=pubkey_file]` |
| `sqlite`     | SQLite provided by [SQLx](https://github.com/launchbadge/sqlx)                                         | `sqlite://?<memory=true>[&file=<database_file>]`                                                                          |
| `sqlserver`  | SQL Server provided by [Tiberius](https://github.com/prisma/tiberius)                                  | `sqlserver://<user>[:<password>]@<host>[:<port>]/<database>`                                                              |

### Examples

Show the available drivers:

```text
.drivers
```

### Demonstration

![](./demo.gif)
