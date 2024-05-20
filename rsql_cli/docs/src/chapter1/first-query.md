## First Query

The first query is a simple one. It selects the database version and returns the result.

### DuckDB

```shell
rsql --url 'duckdb://?memory=true' -- "SELECT version();"
```

### LibSQL

```shell
rsql --url 'libsql://?memory=true' -- "SELECT sqlite_version();"
```

### MariaDB

```shell
rsql --url 'mariadb://<user>[:<password>]@<host>[:<port>]/<database>' -- "SELECT version();"
```

### MySQL

```shell
rsql --url 'mysql://<user>[:<password>]@<host>[:<port>]/<database>' -- "SELECT version();"
```

### Postgres

```shell
rsql --url 'postgres://?embedded=true' -- "SELECT version();"
```

### PostgreSQL

```shell
rsql --url 'postgresql://?embedded=true' -- "SELECT version();"
```

### Rusqlite

```shell
rsql --url 'rusqlite://?memory=true' -- "SELECT sqlite_version();"
```

### Sqlite

```shell
rsql --url 'sqlite://?memory=true' -- "SELECT sqlite_version();"
```
