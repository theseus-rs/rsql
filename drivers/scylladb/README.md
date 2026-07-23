# rsql_driver_scylladb

[![Documentation](https://docs.rs/rsql_driver_scylladb/badge.svg)](https://docs.rs/rsql_driver_scylladb)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_scylladb.svg)](https://crates.io/crates/rsql_driver_scylladb)
[![License](https://img.shields.io/crates/l/rsql_driver_scylladb)](https://github.com/theseus-rs/rsql#license)

`rsql_driver_scylladb` provides native CQL connectivity for ScyllaDB and Scylla Cloud. It is a native-only driver and
is enabled by rsql's `driver-scylladb` feature.

## Connection URL

```text
scylladb://[<user>:<password>@]<host>[:<port>]/[<keyspace>][?node=<host>:<port>][&datacenter=<dc>][&sslmode=<disable|verify-full>][&ssl_ca=<pem-file>][&ssl_cert=<pem-file>&ssl_key=<pem-file>][&client_route=<connection-id>]
```

Port `9042`, plaintext transport, and no selected keyspace are the defaults. Credentials must contain both a username
and password. URL-encode reserved characters in credentials and file paths.

Additional bootstrap nodes can be supplied by repeating `node`. The optional `datacenter` value makes matching nodes
local and preferred by the ScyllaDB driver's load-balancing policy.

## Direct clusters

```shell
rsql --url "scylladb://localhost:9042/my_keyspace"
rsql --url "scylladb://user:password@node1/my_keyspace?node=node2:9042&node=node3:9042&datacenter=dc1"
```

Bound `?` parameters are prepared and cached. SELECT results are fetched with CQL paging and materialized before they
are returned through the rsql result interface. CQL does not expose a portable affected-row count, so successful
non-query statements report `0` changes.

## Scylla Cloud TLS and mTLS

Use [`sslmode=verify-full`](https://rust-driver.docs.scylladb.com/stable/connecting/tls.html) for direct TLS. Without `ssl_ca`, the platform certificate store is used. Supply a PEM CA
file for a private trust root. `ssl_cert` and `ssl_key` are an optional PEM client-certificate pair for mTLS and must be
provided together.

```shell
rsql --url "scylladb://user:password@cloud.example.com/my_keyspace?sslmode=verify-full"
rsql --url "scylladb://user:password@cloud.example.com/my_keyspace?sslmode=verify-full&ssl_ca=/path/ca.pem&ssl_cert=/path/client.pem&ssl_key=/path/client.key"
```

## Private Client Routes

Repeat `client_route` for each Scylla Cloud connection ID:

```shell
rsql --url "scylladb://user:password@private-endpoint:9042/my_keyspace?client_route=connection-id-a&client_route=connection-id-b"
```

[`Client Routes`](https://rust-driver.docs.scylladb.com/stable/connecting/client-routes.html) support is currently marked unstable by the upstream driver. Upstream Client Routes sessions do not
support TLS, so rsql rejects `sslmode=verify-full`, `ssl_ca`, `ssl_cert`, and `ssl_key` when a `client_route` is present.
All cluster nodes must be reachable through the configured routes.

## Metadata and values

The ScyllaDB cluster is exposed as the current catalog and keyspaces as schemas. Metadata includes tables,
materialized views, columns, partition and clustering primary keys, and secondary indexes. Scalar CQL types,
collections, vectors, tuples, and user-defined types are converted to rsql values. Values wider than rsql's native
numeric or temporal range are preserved as strings.

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
