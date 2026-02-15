# rsql_driver_tsv

[![Documentation](https://docs.rs/rsql_driver_tsv/badge.svg)](https://docs.rs/rsql_driver_tsv)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_tsv.svg)](https://crates.io/crates/rsql_driver_tsv)
[![License](https://img.shields.io/crates/l/rsql_driver_tsv)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_tsv` is a data driver for Tab Separated Values (TSV) files.

## Usage

Driver url format: `tsv://<file>[?has_header=<true|false>][&quote=<char>][&skip_rows=<n>]`

The driver is implemented using [Polars SQL](https://docs.pola.rs/user-guide/sql).

### Driver Configuration

| Parameter                | Description                                                                                                   | Default |
|--------------------------|---------------------------------------------------------------------------------------------------------------|---------|
| `has_header`             | Whether the file has a header row.                                                                            | `true`  |
| `separator`              | The character used to separate fields in the file.                                                            | `,`     |
| `quote`                  | The character used to quote fields in the file.                                                               | `"`     |
| `eol`                    | The character used to separate lines in the file.                                                             | `\n`    |
| `skip_rows`              | The number of rows to skip before reading the data.                                                           | `0`     |
| `skip_rows_after_header` | The number of rows to skip after the header.                                                                  | `0`     |
| `truncate_ragged_lines`  | Whether to truncate lines that are longer than the schema.                                                    | `false` |
| `infer_schema_length`    | The number of rows to use when inferring the schema.                                                          | `100`   |
| `ignore_errors`          | Whether to ignore errors. If `true`, errors will be ignored. If `false`, errors will cause the query to fail. | `false` |

## Safety

These crates use `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
