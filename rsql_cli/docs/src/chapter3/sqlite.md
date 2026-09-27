# SQLite (SQLx)

The `sqlite` driver opens a local SQLite (SQLx) database or a temporary in-memory database.

## URL format

```text
sqlite://[<file>]
```

## Options

| Option      | Behavior / default                                                                                                                     |
|-------------|----------------------------------------------------------------------------------------------------------------------------------------|
| `mode`      | `ro` for read-only or `memory` for a named in-memory database. `rw` and `rwc` are also parsed; rsql enables creation of missing files. |
| `cache`     | `private` or `shared`; controls SQLite page-cache sharing.                                                                             |
| `immutable` | `true`/`1` or `false`/`0`; treat the database file as immutable.                                                                       |
| `vfs`       | SQLite virtual filesystem name.                                                                                                        |

## Examples

```shell
rsql --url 'sqlite://' -- 'SELECT 42;'
rsql --url 'sqlite://example.sqlite'
```

## Usage notes

Omit the filename for an in-memory database, whose data lasts for the session. A file path creates
or opens a persistent database; use three slashes for an absolute Unix path. No database server is
required. Use `.tables`, `.views`, and `.describe` to inspect objects.

URL query options apply to file/named database URLs; a bare `sqlite://` opens the default in-memory
database.
