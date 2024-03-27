## First Query

The first query is a simple one. It selects the database version and returns the result.

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
