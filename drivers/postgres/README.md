# `rsql_driver_postgres`

[![Documentation](https://docs.rs/rsql_driver_postgres/badge.svg)](https://docs.rs/rsql_driver_postgres)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_postgres.svg)](https://crates.io/crates/rsql_driver_postgres)
[![License](https://img.shields.io/crates/l/rsql_driver_postgres)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_postgres` connects rsql to PostgreSQL using `tokio-postgres`. It supports existing
servers and managed local instances through `postgresql_embedded`.

## Usage

Driver URL format:

```text
postgres://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
```

```shell
rsql --url 'postgres://postgres@localhost:5432/example?sslmode=disable' -- 'SELECT version();'
rsql --url 'postgres://?embedded=true' -- 'SELECT 42;'
```

The default port is `5432`. This implementation uses `NoTls`; use
[rsql_driver_postgresql](https://docs.rs/rsql_driver_postgresql) for TLS connections.

With `embedded=true`, the driver downloads and starts a managed PostgreSQL instance, creates the
`embedded` database, and stops the server when the connection closes. Initial setup requires
network access unless the PostgreSQL installation is already cached.

## Features

| Feature                | Description                                                                                              | Enabled by default |
|------------------------|----------------------------------------------------------------------------------------------------------|--------------------|
| `tls-native-tls`       | Use platform-native TLS for embedded PostgreSQL downloads; database connections use `NoTls`.             | Yes                |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider for embedded PostgreSQL downloads; database connections use `NoTls`. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider for embedded PostgreSQL downloads; database connections use `NoTls`. | No                 |
