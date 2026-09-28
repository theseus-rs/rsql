# Arrow IPC

The `arrow` driver reads Arrow IPC files and exposes their data through SQL.

## URL format

```text
arrow://<file>
```

## Options

This driver has no format-specific URL query options. The file supplies its schema.

## Examples

```shell
rsql --url 'arrow://people.arrow' -- 'SELECT * FROM people LIMIT 10;'
```

## Usage notes

Use a relative path such as `arrow://people.arrow` or an absolute path such as
`arrow:///data/people.arrow`. Percent-encode spaces and URL delimiters in filenames.

The file is loaded into an in-memory Polars SQL context. Its table name is the filename before the
first dot: `people.csv` becomes `people`. Use `.tables` and `.describe people` to inspect the
imported data. Query results and session changes do not write back to the source file.
