<p align="center"><img width="250" height="250" src="rsql_cli/resources/rsql.png"></p>

# rsql

[![ci](https://github.com/theseus-rs/rsql/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/theseus-rs/rsql/actions/workflows/ci.yml)
[![Documentation](https://docs.rs/rsql_core/badge.svg)](https://docs.rs/rsql_core)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Benchmarks](https://img.shields.io/badge/%F0%9F%90%B0_bencher-enabled-6ec241)](https://bencher.dev/perf/theseus-rs-rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_cli.svg)](https://crates.io/crates/rsql_cli)
[![Github All Releases](https://img.shields.io/github/downloads/theseus-rs/rsql/total.svg)](https://theseus-rs.github.io/rsql/rsql_cli/)
[![License](https://img.shields.io/crates/l/rsql_cli)](https://github.com/theseus-rs/rsql_cli#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql` is a command line interface for databases.  `rsql` is a modern, feature-rich, and user-friendly database client,
that has been designed to be easy to use, and to provide a consistent experience across all supported databases. The
project aims to provide reusable components for building other database clients.

## Getting Started

`rsql` can be installed using the following methods:

### Linux / MacOS

```shell
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/theseus-rs/rsql/releases/latest/download/rsql_cli-installer.sh | sh
```

### Windows

```shell
irm https://github.com/theseus-rs/rsql/releases/latest/download/rsql_cli-installer.ps1 | iex
```

For more information, and additional installations instructions (cargo, homebrew, msi),
visit the [rsql](https://theseus-rs.github.io/rsql/rsql_cli/) site.

![](./rsql_cli/resources/demo.gif)

## Features

| Feature             |                                                                                                 |
|---------------------|-------------------------------------------------------------------------------------------------|
| Databases           | DuckDB, LibSQL (Turso), MariaDB, MySQL, PostgreSQL, Snowflake, SQLite3, SQL Server              |
| Embedded PostgreSQL | ✅                                                                                               |
| Syntax Highlighting | ✅                                                                                               |
| Result Highlighting | ✅                                                                                               |
| History             | ✅                                                                                               |
| SQL File Execution  | ✅                                                                                               |
| Output Formats      | ascii, csv, expanded, html, json, jsonl, markdown, plain, psql, sqlite, tsv, unicode, xml, yaml |
| Localized Interface | 40+ languages¹                                                                                  |
| Key Bindings        | emacs, vi                                                                                       |

¹ Computer translations; human translations welcome

## Usage

### Interactive Mode

```shell
rsql --url "<url>"
```

### Running a single Query

```shell
rsql --url "<url>" -- "<query>"
```

| Driver            | URL                                                                                                                       |
|-------------------|---------------------------------------------------------------------------------------------------------------------------|
| duckdb            | `duckdb://?<memory=true>[&file=<database_file>]`                                                                          |
| libsql¹           | `libsql://<host>?[<memory=true>][&file=<database_file>][&auth_token=<token>]`                                             |
| mariadb (sqlx)    | `mariadb://<user>[:<password>]@<host>[:<port>]/<database>`                                                                |
| mysql (sqlx)      | `mysql://<user>[:<password>]@<host>[:<port>]/<database>`                                                                  |
| postgres          | `postgres://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                               |
| postgresql (sqlx) | `postgresql://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                             |
| redshift (sqlx)   | `redshift://<user[:password>]@<host>[:<port>]/<database>`                                                                 |
| rusqlite          | `rusqlite://?<memory=true>[&file=<database_file>]`                                                                        |
| snowflake         | `snowflake://<user>[:<token>]@<account>.snowflakecomputing.com/[?private_key_file=pkey_file&public_key_file=pubkey_file]` |
| sqlite (sqlx)     | `sqlite://?<memory=true>[&file=<database_file>]`                                                                          |
| sqlserver         | `sqlserver://<user>[:<password>]@<host>[:<port>]/<database>`                                                              |

¹ `libsql` needs to be enabled with the `libsql` feature flag; it is disabled by default as it conflicts
with `rusqlite`.

## Safety

These crates use `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

<a href="https://vscode.dev/redirect?url=vscode://ms-vscode-remote.remote-containers/cloneInVolume?url=https://github.com/theseus-rs/rsql">
<img
  src="https://img.shields.io/static/v1?label=VSCode%20Development%20Container&logo=visualstudiocode&message=Open&color=orange"
  alt="VSCode Development Container"
/>
</a>
<br/>
<a href="https://github.dev/theseus-rs/rsql">
<img
  src="https://img.shields.io/static/v1?label=GitHub%20Codespaces&logo=github&message=Open&color=orange"
  alt="GitHub Codespaces"
/>
</a>
