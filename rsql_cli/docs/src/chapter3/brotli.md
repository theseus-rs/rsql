# Brotli

The `brotli` driver decompresses a local Brotli file and opens the contents with the matching data
driver.

## URL format

```text
brotli://<file>[?<format-options>]
```

## Options

There are no decompression-specific query options. Query parameters are forwarded to the detected
data driver; for example, CSV accepts `has_header`, `quote`, and `skip_rows`.

## Examples

```shell
rsql --url 'brotli://people.csv.br?quote=%22' -- 'SELECT * FROM people;'
```

## Usage notes

Keep the original extension before `.br` so the decompressed filename identifies the data format.
The corresponding format driver must be enabled. For CSV content, see [CSV options](csv.md).
Decompressed data is staged in a temporary directory; queries do not rewrite the compressed source.

Use `.tables` to inspect the resulting tables.
