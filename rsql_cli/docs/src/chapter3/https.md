# HTTPS

The `https` driver downloads a resource over HTTPS, detects its format, and opens it with the
matching data driver.

## URL format

```text
https://<host>[:<port>]/<path>[?_headers=<headers>]
```

## Options

| Option                   | Behavior / default                                                                                                                |
|--------------------------|-----------------------------------------------------------------------------------------------------------------------------------|
| `_headers`               | Semicolon-separated `name=value` request headers, percent-encoded as one query value. For example, `Accept%3Dapplication%2Fjson`. |
| `Other query parameters` | Used as request query parameters and headers, and forwarded to the detected data driver.                                          |

## Examples

```shell
rsql --url 'https://example.com/people.csv' -- 'SELECT * FROM people LIMIT 10;'
rsql --url 'https://example.com/people.json?_headers=Accept%3Dapplication%2Fjson'
```

## Usage notes

The response content type and filename select the data driver, which must be enabled. The resource
is downloaded into a temporary directory. Use `.tables` to inspect the imported tables and the
request/response header tables.

Query options, including expanded `_headers` values, are also included in the outgoing request URL.
Avoid placing secret header values in this URL. Format options are described on the corresponding
driver page, such as [JSON](json.md).
