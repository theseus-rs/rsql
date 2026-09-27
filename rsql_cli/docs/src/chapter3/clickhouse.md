# ClickHouse

The `clickhouse` driver connects to ClickHouse through its HTTP interface.

## URL format

```text
clickhouse://[<user>[:<password>]@]<host>[:<port>]/[<database>][?<options>]
```

## Options

| Option         | Behavior / default                                                                 |
|----------------|------------------------------------------------------------------------------------|
| `scheme`       | HTTP transport scheme; default `https`. Use `http` for a local plaintext endpoint. |
| `access_token` | Access token used by the ClickHouse client.                                        |

## Examples

```shell
rsql --url 'clickhouse://default@localhost:8123/default?scheme=http' -- 'SELECT version();'
rsql --url 'clickhouse://user:password@db.example.com:8443/default?scheme=https'
```

## Usage notes

The driver defaults to host `localhost` and port `8123`, including when using HTTPS. Set the port
appropriate for your service. The path selects the database. Successful non-query statements report
`0` changes. Use `.tables` and `.describe` to inspect the database.
