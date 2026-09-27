# LZ4

The `lz4` driver decompresses a local LZ4 file and opens the contents with the matching data driver.

## URL format

```text
lz4://<file>[?<format-options>]
```

## Options

There are no decompression-specific query options. Query parameters are forwarded to the detected
data driver; for example, CSV accepts `has_header`, `quote`, and `skip_rows`.

## Examples

```shell
rsql --url 'lz4://people.csv.lz4?quote=%22' -- 'SELECT * FROM people;'
```

## Usage notes

Keep the original extension before `.lz4` so the decompressed filename identifies the data format.
The corresponding format driver must be enabled. For CSV content, see [CSV options](csv.md).
Decompressed data is staged in a temporary directory; queries do not rewrite the compressed source.

Use `.tables` to inspect the resulting tables.
