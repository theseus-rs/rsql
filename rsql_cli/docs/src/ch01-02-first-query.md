## First Query

The first query is a simple one. It selects the database version and returns the result.

### PostgreSQL

```shell
rsql --url postgresql::embedded: -- "SELECT version();"
```

### Sqlite

```shell
rsql --url sqlite::memory: -- "SELECT sqlite_version();"
```
