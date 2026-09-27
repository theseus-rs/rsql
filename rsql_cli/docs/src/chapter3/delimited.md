# Delimited text

The `delimited` driver loads separated text fields into Polars SQL.

## URL format

```text
delimited://<file>[?has_header=<true|false>&quote=<char>&skip_rows=<rows>]
```

## Options

| Option                   | Behavior / default                                                                          |
|--------------------------|---------------------------------------------------------------------------------------------|
| `separator`              | Single ASCII field separator; default comma. Use `%09` for tab or `%7C` for a vertical bar. |
| `has_header`             | Use the first row as column names; default `true`.                                          |
| `skip_rows`              | Rows to skip before reading the header/data; default `0`.                                   |
| `skip_rows_after_header` | Rows to skip after the header; default `0`.                                                 |
| `quote`                  | Single ASCII quote character. Quoting is disabled by default; use `%22` for double quotes.  |
| `eol`                    | Single ASCII line terminator; default newline (`%0A`).                                      |
| `truncate_ragged_lines`  | Truncate rows wider than the inferred schema when `true`; default `false`.                  |
| `infer_schema_length`    | Rows used to infer column types; default `100`. Set `0` to examine all rows.                |
| `ignore_errors`          | Ignore reader conversion errors when `true`; default `false`.                               |

## Examples

```shell
rsql --url 'delimited://people.txt?separator=%7C&quote=%22' -- 'SELECT * FROM people;'
```

## Usage notes

Choose the field separator with `separator`. Character options require exactly one ASCII character
after URL decoding.

Use a relative path such as `delimited://people.txt` or an absolute path such as
`delimited:///data/people.txt`. Percent-encode spaces and URL delimiters in filenames.

The file is loaded into an in-memory Polars SQL context. Its table name is the filename before the
first dot: `people.csv` becomes `people`. Use `.tables` and `.describe people` to inspect the
imported data. Query results and session changes do not write back to the source file.
