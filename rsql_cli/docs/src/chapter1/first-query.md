## First Query

The following examples show how to run a simple query using the `rsql` CLI tool for different data sources.

### CockroachDB

```shell
rsql --url "cockroachdb://<user[:password>]@<host>[:<port>]/<database>" -- "SELECT version();"
```

### DuckDB

```shell
rsql --url "duckdb://" -- "SELECT version();"
```

### LibSQL

```shell
rsql --url "libsql://?memory=true" -- "SELECT sqlite_version();"
```

### MariaDB

```shell
rsql --url "mariadb://<user>[:<password>]@<host>[:<port>]/<database>" -- "SELECT version();"
```

### MySQL

```shell
rsql --url "mysql://<user>[:<password>]@<host>[:<port>]/<database>" -- "SELECT version();"
```

### Postgres

```shell
rsql --url "postgres://?embedded=true" -- "SELECT version();"
```

### PostgreSQL

```shell
rsql --url "postgresql://?embedded=true" -- "SELECT version();"
```

### Redshift

```shell
rsql --url "redshift://<user[:password>]@<host>[:<port>]/<database>" -- "SELECT version();"
```

### Rusqlite

```shell
rsql --url "rusqlite://" -- "SELECT sqlite_version();"
```

### Snowflake

```shell
rsql --url "snowflake://<user>@<account>.snowflakecomputing.com/[?private_key_file=pkey_file&public_key_file=pubkey_file]" -- "SELECT CURRENT_VERSION();"
```

or

```shell
rsql --url "snowflake://<user>[:<token>]@<account>.snowflakecomputing.com/" -- "SELECT CURRENT_VERSION();"
```

### Sqlite

```shell
rsql --url "sqlite://" -- "SELECT sqlite_version();"
```
