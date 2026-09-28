# ScyllaDB

The `scylladb` driver provides native CQL connections to ScyllaDB and Scylla Cloud.

## URL format

```text
scylladb://[<user>:<password>@]<host>[:<port>]/[<keyspace>][?<options>]
```

## Options

| Option               | Behavior / default                                                       |
|----------------------|--------------------------------------------------------------------------|
| `node`               | Additional bootstrap node as `host:port`; may be repeated.               |
| `datacenter`         | Prefer nodes in this datacenter.                                         |
| `sslmode`            | `disable` (default) or `verify-full` for TLS with hostname verification. |
| `ssl_ca`             | PEM CA file; defaults to the platform trust store.                       |
| `ssl_cert / ssl_key` | PEM client certificate and key for mTLS; supply both together.           |
| `client_route`       | Scylla Cloud connection ID for a Private Client Route; may be repeated.  |

## Examples

```shell
rsql --url 'scylladb://localhost:9042/my_keyspace'
rsql --url 'scylladb://user:password@node1/my_keyspace?node=node2:9042&datacenter=dc1'
rsql --url 'scylladb://user:password@cloud.example.com/my_keyspace?sslmode=verify-full'
```

## Usage notes

Port `9042`, plaintext transport, and no selected keyspace are the defaults. Credentials require
both a username and password. URL-encode reserved characters in credentials and paths.

Private Client Routes currently cannot be combined with TLS options. All nodes must be reachable
through the configured routes.

Use CQL for queries. Bound parameters use `?`; successful non-query statements report `0` changes.
Metadata exposes the cluster as a catalog, keyspaces as schemas, and tables, materialized views,
columns, primary keys, and indexes. This driver is available on native targets.
