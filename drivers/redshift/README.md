# `rsql_driver_redshift`

[![Documentation](https://docs.rs/rsql_driver_redshift/badge.svg)](https://docs.rs/rsql_driver_redshift)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_redshift.svg)](https://crates.io/crates/rsql_driver_redshift)
[![License](https://img.shields.io/crates/l/rsql_driver_redshift)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_redshift` connects rsql to Amazon Redshift through the PostgreSQL protocol. It uses
[rsql_driver_postgresql](https://docs.rs/rsql_driver_postgresql) for connections and exposes the
Redshift SQL dialect.

## Usage

Driver URL format:

```text
redshift://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
```

```shell
rsql --url 'redshift://user:password@cluster.example.com:5439/example?sslmode=verify-full' -- 'SELECT 1;'
```

Specify the Redshift service port explicitly; the underlying PostgreSQL connection defaults to
`5432`. PostgreSQL connection options such as `sslmode`, `sslrootcert`, `sslcert`, and `sslkey` are
passed through to `SQLx`.

## Features

| Feature                | Description                                                        | Enabled by default |
|------------------------|--------------------------------------------------------------------|--------------------|
| `tls-native-tls`       | Use platform-native TLS through the PostgreSQL driver.             | Yes                |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider through the PostgreSQL driver. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider through the PostgreSQL driver. | No                 |
