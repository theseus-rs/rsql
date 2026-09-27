# Fixed-width text

The `fwf` driver splits text rows into fixed-width columns and queries them with Polars SQL.

## URL format

```text
fwf://<file>?widths=<widths>[&headers=<names>]
```

## Options

| Option    | Behavior / default                                                                                  |
|-----------|-----------------------------------------------------------------------------------------------------|
| `widths`  | Required comma-separated field widths, such as `5,20,3`. Widths are measured in bytes.              |
| `headers` | Comma-separated column names. The number must match `widths`; defaults to `A`, `B`, `C`, and so on. |

## Examples

```shell
rsql --url 'fwf://people.fwf?widths=5,20,3&headers=id,name,age' -- 'SELECT * FROM people;'
```

## Usage notes

Every line is treated as data. Fields are trimmed and imported as strings; cast columns in SQL when
needed. Rows shorter than the configured widths fail to load. There are no `has_header` or
`skip_rows` options.

The file is loaded into an in-memory Polars SQL context. Its table name is the filename before the
first dot: `people.csv` becomes `people`. Use `.tables` and `.describe people` to inspect the
imported data. Query results and session changes do not write back to the source file.
