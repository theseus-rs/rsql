# `rsql_driver_postgresql`

[![Documentation](https://docs.rs/rsql_driver_postgresql/badge.svg)](https://docs.rs/rsql_driver_postgresql)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_postgresql.svg)](https://crates.io/crates/rsql_driver_postgresql)
[![License](https://img.shields.io/crates/l/rsql_driver_postgresql)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_postgresql` connects rsql to PostgreSQL using `SQLx`. It supports existing servers
and managed local instances through `postgresql_embedded`.

## Usage

Driver URL format:

```text
postgresql://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
```

```shell
rsql --url 'postgresql://user:password@localhost:5432/example' -- 'SELECT 1;'
rsql --url 'postgresql://?embedded=true' -- 'SELECT version();'
```

The default port is `5432`. Connection options include `sslmode`, `sslrootcert`, `sslcert`,
`sslkey`, and `application_name`.

With `embedded=true`, the driver downloads and starts a managed PostgreSQL instance, creates the
`embedded` database, and stops the server when the connection closes. Initial setup requires
network access unless the PostgreSQL installation is already cached.

## Features

| Feature                | Description                                                                                   | Enabled by default |
|------------------------|-----------------------------------------------------------------------------------------------|--------------------|
| `tls-native-tls`       | Use platform-native TLS for PostgreSQL connections and embedded server downloads.             | Yes                |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider for PostgreSQL connections and embedded server downloads. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider for PostgreSQL connections and embedded server downloads. | No                 |
