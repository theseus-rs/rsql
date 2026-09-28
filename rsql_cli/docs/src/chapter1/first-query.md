## First Query

The following examples show how to run a simple query using the `rsql` CLI tool for different data
sources. Replace placeholders (e.g., `<user>`, `<host>`, `<database>` ) with your actual connection
details.

### CockroachDB

```shell
rsql --url "cockroachdb://<user[:password]>@<host>[:<port>]/<database>" -- "SELECT version();"
```

### DuckDB (in-memory or file)

```shell
# In-memory
rsql --url "duckdb://" -- "SELECT version();"
# File-based
rsql --url "duckdb:///path/to/file.duckdb" -- "SELECT COUNT(*) FROM my_table;"
```

### H2 (in-memory or file)

```shell
rsql --url "h2://" -- "SELECT H2VERSION();"
rsql --url "h2:./example" -- "SELECT 42;"
```

The H2 driver downloads and caches H2 2.5.252 and its runtime dependencies from Maven Central.
`h2://` creates a private in-memory database; a file URL retains data between sessions. See the [H2
guide](../chapter3/h2.md) for options.

### JDBC

Resolve the JDBC driver and its runtime dependencies from Maven Central:

```shell
rsql --url "jdbc:postgresql://localhost/example?user=postgres&dependency=org.postgresql:postgresql:42.7.13&driver=org.postgresql.Driver" -- "SELECT version();"
```

Omit `driver` to use JDBC service discovery. See [JDBC](../chapter3/jdbc.md) and
[H2](../chapter3/h2.md) for URL options, Java runtime setup, and offline caching.

### MariaDB

```shell
rsql --url "mariadb://<user>[:<password>]@<host>[:<port>]/<database>" -- "SELECT version();"
```

### MySQL

```shell
rsql --url "mysql://<user>[:<password>]@<host>[:<port>]/<database>" -- "SELECT version();"
```

### Postgres (embedded or remote)

```shell
rsql --url "postgres://?embedded=true" -- "SELECT version();"
rsql --url "postgres://<user>:<password>@<host>:<port>/<database>" -- "SELECT COUNT(*) FROM my_table;"
```

### PostgreSQL (embedded or remote)

```shell
rsql --url "postgresql://?embedded=true" -- "SELECT version();"
rsql --url "postgresql://<user>:<password>@<host>:<port>/<database>" -- "SELECT COUNT(*) FROM my_table;"
```

### Redshift

```shell
rsql --url "redshift://<user[:password]>@<host>[:<port>]/<database>" -- "SELECT version();"
```

### Rusqlite

```shell
rsql --url "rusqlite://" -- "SELECT sqlite_version();"
```

### ScyllaDB

```shell
rsql --url "scylladb://<user>:<password>@<host>:9042/<keyspace>" -- "SELECT release_version FROM system.local;"
```

### Snowflake

```shell
rsql --url "snowflake://<user>@<account>.snowflakecomputing.com/[?private_key_file=pkey_file&public_key_file=pubkey_file]" -- "SELECT CURRENT_VERSION();"
# Or with token
rsql --url "snowflake://<user>[:<token>]@<account>.snowflakecomputing.com/" -- "SELECT CURRENT_VERSION();"
```

### Querying Data Files (CSV, Parquet, etc.)

```shell
rsql --url "csv:///path/to/data.csv" -- "SELECT * FROM data LIMIT 5;"
rsql --url "parquet:///path/to/data.parquet" -- "SELECT column1, column2 FROM data WHERE column3 > 100;"
```

### Sqlite

```shell
rsql --url "sqlite://" -- "SELECT sqlite_version();"
```

### Tips

- Use the `--format` option or `.format` command to change output format (e.g., CSV, JSON).
- Use `.help` for a list of available commands.
- For more advanced examples, see the [FAQ & Tips](../appendix/index.md#tips--tricks).
