# File detection

The `file` driver detects a local file’s type and selects a matching driver.

## URL format

```text
file://<file>[?<format-options>]
```

## Options

There are no detection-specific query options. Options are forwarded to the detected driver. For
example, see [CSV](csv.md), [Excel](excel.md), and [SQLite](sqlite.md).

## Examples

```shell
rsql --url 'file://people.csv?quote=%22' -- 'SELECT * FROM people;'
rsql --url 'file:///data/example.sqlite'
```

## Usage notes

Use relative or absolute paths. Detection requires the file to exist and a matching driver to be
enabled. Choose an explicit format URL when detection cannot distinguish the content. Compressed
files are delegated through the matching decompression driver. Use `.tables` to inspect the opened
data.
