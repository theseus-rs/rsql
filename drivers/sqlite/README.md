# `rsql_driver_sqlite`

[![Documentation](https://docs.rs/rsql_driver_sqlite/badge.svg)](https://docs.rs/rsql_driver_sqlite)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_sqlite.svg)](https://crates.io/crates/rsql_driver_sqlite)
[![License](https://img.shields.io/crates/l/rsql_driver_sqlite)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_sqlite` connects rsql to local SQLite databases using `SQLx`.

## Usage

Driver URL format: `sqlite://[<file>][?<options>]`

```shell
rsql --url 'sqlite://' -- 'SELECT 42;'
rsql --url 'sqlite://example.sqlite'
rsql --url 'sqlite://example.sqlite?mode=ro'
```

Omit the filename to open a temporary in-memory database. A file path opens or creates a persistent
database; use three slashes for an absolute Unix path, such as `sqlite:///path/example.sqlite`.
No separate database server is required. File and named database URLs support options including
`mode`, `cache`, `immutable`, and `vfs`; a bare `sqlite://` uses the default in-memory database.

## Features

| Feature                | Description                                                                        | Enabled by default |
|------------------------|------------------------------------------------------------------------------------|--------------------|
| `tls-native-tls`       | Enable platform-native TLS in `SQLx`; SQLite connections remain local.             | Yes                |
| `tls-rustls-aws-lc-rs` | Enable Rustls with the AWS-LC provider in `SQLx`; SQLite connections remain local. | No                 |
| `tls-rustls-ring`      | Enable Rustls with the `ring` provider in `SQLx`; SQLite connections remain local. | No                 |
