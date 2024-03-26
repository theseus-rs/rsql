## drivers

### Usage

```text
.drivers
```

### Description

The drivers command displays the available database drivers.

| Driver       | Description                                                                                     | URL                                                                           |
|--------------|-------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------|
| `postgresql` | PostgreSQL driver provided by [SQLx](https://github.com/launchbadge/sqlx)                       | `postgresql://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>` |
| `rusqlite`   | SQLite provided by [Rusqlite](https://github.com/rusqlite/rusqlite?tab=readme-ov-file#rusqlite) | `rusqlite://?<memory=true>[&file=<database_file>]`                            |
| `sqlite`     | SQLite provided by [SQLx](https://github.com/launchbadge/sqlx)                                  | `sqlite://?<memory=true>[&file=<database_file>]`                              |

### Examples

Show the available drivers:

```text
.drivers
```
