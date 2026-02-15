# rsql_driver_json

[![Documentation](https://docs.rs/rsql_driver_json/badge.svg)](https://docs.rs/rsql_driver_json)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_json.svg)](https://crates.io/crates/rsql_driver_json)
[![License](https://img.shields.io/crates/l/rsql_driver_json)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_json` is a data driver for JSON files.

## Usage

Driver url format: `json://<file>`

The driver is implemented using [Polars SQL](https://docs.pola.rs/user-guide/sql).

## Safety

These crates use `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
