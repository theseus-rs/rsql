# Avro

The `avro` driver reads Avro files and exposes their data through SQL.

## URL format

```text
avro://<file>
```

## Options

This driver has no format-specific URL query options. The file supplies its schema.

## Examples

```shell
rsql --url 'avro://people.avro' -- 'SELECT * FROM people LIMIT 10;'
```

## Usage notes

Use a relative path such as `avro://people.avro` or an absolute path such as
`avro:///data/people.avro`. Percent-encode spaces and URL delimiters in filenames.

The file is loaded into an in-memory Polars SQL context. Its table name is the filename before the
first dot: `people.csv` becomes `people`. Use `.tables` and `.describe people` to inspect the
imported data. Query results and session changes do not write back to the source file.
