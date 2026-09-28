# ORC

The `orc` driver reads ORC files and exposes their data through SQL.

## URL format

```text
orc://<file>
```

## Options

This driver has no format-specific URL query options. The file supplies its schema.

## Examples

```shell
rsql --url 'orc://people.orc' -- 'SELECT * FROM people LIMIT 10;'
```

## Usage notes

Use a relative path such as `orc://people.orc` or an absolute path such as `orc:///data/people.orc`.
Percent-encode spaces and URL delimiters in filenames.

The file is loaded into an in-memory Polars SQL context. Its table name is the filename before the
first dot: `people.csv` becomes `people`. Use `.tables` and `.describe people` to inspect the
imported data. Query results and session changes do not write back to the source file.
