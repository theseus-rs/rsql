# CSV

The `csv` driver loads separated text fields into Polars SQL.

## URL format

```text
csv://<file>[?has_header=<true|false>&quote=<char>&skip_rows=<rows>]
```

## Options

| Option                   | Behavior / default                                                                         |
|--------------------------|--------------------------------------------------------------------------------------------|
| `has_header`             | Use the first row as column names; default `true`.                                         |
| `skip_rows`              | Rows to skip before reading the header/data; default `0`.                                  |
| `skip_rows_after_header` | Rows to skip after the header; default `0`.                                                |
| `quote`                  | Single ASCII quote character. Quoting is disabled by default; use `%22` for double quotes. |
| `eol`                    | Single ASCII line terminator; default newline (`%0A`).                                     |
| `truncate_ragged_lines`  | Truncate rows wider than the inferred schema when `true`; default `false`.                 |
| `infer_schema_length`    | Rows used to infer column types; default `100`. Set `0` to examine all rows.               |
| `ignore_errors`          | Ignore reader conversion errors when `true`; default `false`.                              |

## Examples

```shell
rsql --url 'csv://people.csv?quote=%22' -- 'SELECT * FROM people LIMIT 10;'
```

## Usage notes

The field separator is fixed to comma.

Use a relative path such as `csv://people.csv` or an absolute path such as `csv:///data/people.csv`.
Percent-encode spaces and URL delimiters in filenames.

The file is loaded into an in-memory Polars SQL context. Its table name is the filename before the
first dot: `people.csv` becomes `people`. Use `.tables` and `.describe people` to inspect the
imported data. Query results and session changes do not write back to the source file.
