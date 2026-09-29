# `rsql_driver_duckdb`

[![Documentation](https://docs.rs/rsql_driver_duckdb/badge.svg)](https://docs.rs/rsql_driver_duckdb)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_duckdb.svg)](https://crates.io/crates/rsql_driver_duckdb)
[![License](https://img.shields.io/crates/l/rsql_driver_duckdb)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_duckdb` connects rsql to local `DuckDB` databases using the `duckdb` crate with a
bundled database engine.

## Usage

Driver URL format: `duckdb://[<file>]`

```shell
rsql --url 'duckdb://' -- 'SELECT 42;'
rsql --url 'duckdb://example.duckdb'
```

Omit the filename to open a temporary in-memory database. A file path opens or creates a persistent
database; use three slashes for an absolute Unix path, such as `duckdb:///path/example.duckdb`.
No separate database server is required. There are no driver-specific URL query options; configure
the database through SQL settings or pragmas.
