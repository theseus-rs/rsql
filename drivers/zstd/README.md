# rsql_driver_zstd

[![ci](https://github.com/theseus-rs/rsql/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/theseus-rs/rsql/actions/workflows/ci.yml)
[![Documentation](https://docs.rs/rsql_driver_http/badge.svg)](https://docs.rs/rsql_driver_zstd)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Benchmarks](https://img.shields.io/badge/%F0%9F%90%B0_bencher-enabled-6ec241)](https://bencher.dev/perf/theseus-rs-rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_zstd.svg)](https://crates.io/crates/rsql_driver_zstd)
[![License](https://img.shields.io/crates/l/rsql_driver_zstd)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_zstd` is a zstd decompression driver.

## Usage

Driver url format: `zstd://<file>`

## Safety

These crates use `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
