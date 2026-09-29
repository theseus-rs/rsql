# `rsql_driver_rusqlite`

[![Documentation](https://docs.rs/rsql_driver_rusqlite/badge.svg)](https://docs.rs/rsql_driver_rusqlite)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_rusqlite.svg)](https://crates.io/crates/rsql_driver_rusqlite)
[![License](https://img.shields.io/crates/l/rsql_driver_rusqlite)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_rusqlite` connects rsql to local SQLite databases using `rusqlite` with a bundled
SQLite engine.

## Usage

Driver URL format: `rusqlite://[<file>]`

```shell
rsql --url 'rusqlite://' -- 'SELECT 42;'
rsql --url 'rusqlite://example.sqlite'
```

Omit the filename to open a temporary in-memory database. A file path opens or creates a persistent
database; use three slashes for an absolute Unix path, such as `rusqlite:///path/example.sqlite`.
No separate database server is required. Configure the database through SQL pragmas; this driver
does not expose the `SQLx` SQLite driver's URL query options.
