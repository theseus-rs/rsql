# FlightSQL

The `flightsql` driver connects to an Apache Arrow FlightSQL server.

## URL format

```text
flightsql://[<user>[:<password>]@]<host>[:<port>][?scheme=<http|https>]
```

## Options

| Option   | Behavior / default                                                    |
|----------|-----------------------------------------------------------------------|
| `scheme` | Transport scheme; default `https`. Use `http` for a plaintext server. |

## Examples

```shell
rsql --url 'flightsql://localhost:31337?scheme=http' -- 'SELECT 1;'
rsql --url 'flightsql://user:password@flight.example.com:31337?scheme=https'
```

## Usage notes

A host is required; the default port is `31337`. Supplying a username triggers a FlightSQL
authentication handshake. The server determines supported SQL, catalogs, schemas, and metadata. Use
`.catalogs`, `.schemas`, and `.tables` to explore available data.
