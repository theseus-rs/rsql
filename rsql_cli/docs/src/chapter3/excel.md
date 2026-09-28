# Excel

The `excel` driver reads Excel workbooks and exposes sheets as SQL tables.

## URL format

```text
excel://<file>[?has_header=<true|false>&skip_rows=<rows>]
```

## Options

| Option                   | Behavior / default                                                           |
|--------------------------|------------------------------------------------------------------------------|
| `has_header`             | Use the first row as column names; default `true`.                           |
| `skip_rows`              | Rows to skip before reading the header/data; default `0`.                    |
| `skip_rows_after_header` | Rows to skip after the header; default `0`.                                  |
| `infer_schema_length`    | Rows used to infer column types; default `100`. Set `0` to examine all rows. |
| `ignore_errors`          | Ignore reader conversion errors when `true`; default `false`.                |

## Examples

```shell
rsql --url 'excel://people.xlsx'
rsql --url 'excel://people.xlsx?has_header=false&skip_rows=2'
```

## Usage notes

A workbook with one sheet uses the filename before the first dot as its table name. With multiple
sheets, tables are named `<file>__<sheet>`; non-alphanumeric characters in sheet names become
underscores. Use `.tables` to find the names. Without a header row, columns are named `A`, `B`, and
so on.

Use a relative path such as `excel://people.xlsx` or an absolute path such as
`excel:///data/people.xlsx`. Percent-encode spaces and URL delimiters in filenames.

Sheets are loaded into an in-memory Polars SQL context. Session changes do not update the workbook.
