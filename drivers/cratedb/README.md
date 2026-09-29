# `rsql_driver_cratedb`

[![Documentation](https://docs.rs/rsql_driver_cratedb/badge.svg)](https://docs.rs/rsql_driver_cratedb)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_cratedb.svg)](https://crates.io/crates/rsql_driver_cratedb)
[![License](https://img.shields.io/crates/l/rsql_driver_cratedb)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_cratedb` connects rsql to `CrateDB` through the PostgreSQL protocol. It uses
[rsql_driver_postgresql](https://docs.rs/rsql_driver_postgresql), which is implemented with `SQLx`.

## Usage

Driver URL format:

```text
cratedb://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
```

```shell
rsql --url 'cratedb://user:password@localhost:5432/example' -- 'SELECT 1;'
```

The underlying connection defaults to port `5432`. PostgreSQL connection options such as `sslmode`,
`sslrootcert`, `sslcert`, and `sslkey` are passed through to `SQLx`. SQL features and metadata depend
on the `CrateDB` server.

## Features

Default status follows this crate's `default` feature, including feature aliases.

| Feature                | Description                                                        | Enabled by default |
|------------------------|--------------------------------------------------------------------|--------------------|
| `tls-native-tls`       | Use platform-native TLS through the PostgreSQL driver.             | Yes                |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider through the PostgreSQL driver. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider through the PostgreSQL driver. | No                 |
