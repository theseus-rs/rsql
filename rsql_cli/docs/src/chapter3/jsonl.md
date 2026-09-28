# JSON Lines

The `jsonl` driver reads JSON Lines data and queries it with Polars SQL.

## URL format

```text
jsonl://<file>[?infer_schema_length=<rows>&ignore_errors=<true|false>]
```

## Options

| Option                | Behavior / default                                                           |
|-----------------------|------------------------------------------------------------------------------|
| `infer_schema_length` | Rows used to infer column types; default `100`. Set `0` to examine all rows. |
| `ignore_errors`       | Ignore reader conversion errors when `true`; default `false`.                |

## Examples

```shell
rsql --url 'jsonl://people.jsonl' -- 'SELECT * FROM people LIMIT 10;'
rsql --url 'jsonl://people.jsonl?infer_schema_length=0'
```

## Usage notes

Use one JSON row object per line.

Use a relative path such as `jsonl://people.jsonl` or an absolute path such as
`jsonl:///data/people.jsonl`. Percent-encode spaces and URL delimiters in filenames.

The file is loaded into an in-memory Polars SQL context. Its table name is the filename before the
first dot: `people.csv` becomes `people`. Use `.tables` and `.describe people` to inspect the
imported data. Query results and session changes do not write back to the source file.
