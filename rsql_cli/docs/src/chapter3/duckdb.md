# DuckDB

The `duckdb` driver opens a local DuckDB database or a temporary in-memory database.

## URL format

```text
duckdb://[<file>]
```

## Options

There are no driver-specific URL query options. Configure the database using its SQL settings or
pragmas.

## Examples

```shell
rsql --url 'duckdb://' -- 'SELECT 42;'
rsql --url 'duckdb://example.duckdb'
```

## Usage notes

Omit the filename for an in-memory database, whose data lasts for the session. A file path creates
or opens a persistent database; use three slashes for an absolute Unix path. No database server is
required. Use `.tables`, `.views`, and `.describe` to inspect objects.

Database changes are written to the selected file when using a persistent database.
