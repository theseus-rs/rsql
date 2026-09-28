# JSON

The `json` driver reads JSON data and queries it with Polars SQL.

## URL format

```text
json://<file>[?infer_schema_length=<rows>&ignore_errors=<true|false>]
```

## Options

| Option                | Behavior / default                                                           |
|-----------------------|------------------------------------------------------------------------------|
| `infer_schema_length` | Rows used to infer column types; default `100`. Set `0` to examine all rows. |
| `ignore_errors`       | Ignore reader conversion errors when `true`; default `false`.                |

## Examples

```shell
rsql --url 'json://people.json' -- 'SELECT * FROM people LIMIT 10;'
rsql --url 'json://people.json?infer_schema_length=0'
```

## Usage notes

Use a JSON array of row objects.

Use a relative path such as `json://people.json` or an absolute path such as
`json:///data/people.json`. Percent-encode spaces and URL delimiters in filenames.

The file is loaded into an in-memory Polars SQL context. Its table name is the filename before the
first dot: `people.csv` becomes `people`. Use `.tables` and `.describe people` to inspect the
imported data. Query results and session changes do not write back to the source file.
