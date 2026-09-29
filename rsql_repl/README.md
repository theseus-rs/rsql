# `rsql_repl`

[![Documentation](https://docs.rs/rsql_repl/badge.svg)](https://docs.rs/rsql_repl)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_repl.svg)](https://crates.io/crates/rsql_repl)
[![License](https://img.shields.io/crates/l/rsql_repl)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_repl` is a library for creating a REPL a command line SQL interface.

## Getting Started

## Features

| Feature                | Description                                                                          | Enabled by default |
|------------------------|--------------------------------------------------------------------------------------|--------------------|
| `all`                  | Enable all drivers and output formats.                                               | No                 |
| `all-drivers`          | Enable every database, file, compression, and remote-data driver.                    | No                 |
| `all-formats`          | Enable every output format.                                                          | No                 |
| `driver-arrow`         | Query Arrow IPC files.                                                               | No                 |
| `driver-avro`          | Query Avro files.                                                                    | No                 |
| `driver-brotli`        | Read Brotli compressed data.                                                         | No                 |
| `driver-bzip2`         | Read Bzip2 compressed data.                                                          | No                 |
| `driver-clickhouse`    | Connect to `ClickHouse` through its HTTP interface.                                  | No                 |
| `driver-cockroachdb`   | Connect to `CockroachDB` through the PostgreSQL protocol.                            | No                 |
| `driver-cratedb`       | Connect to `CrateDB` through the PostgreSQL protocol.                                | No                 |
| `driver-csv`           | Query CSV files.                                                                     | No                 |
| `driver-delimited`     | Query delimited text files with a configurable delimiter.                            | No                 |
| `driver-duckdb`        | Query local or in-memory `DuckDB` databases.                                         | No                 |
| `driver-dynamodb`      | Query DynamoDB through `PartiQL`.                                                    | No                 |
| `driver-excel`         | Query Excel workbooks.                                                               | No                 |
| `driver-file`          | Select a driver automatically using file detection.                                  | No                 |
| `driver-flightsql`     | Connect to Arrow Flight SQL servers.                                                 | No                 |
| `driver-fwf`           | Query fixed-width text files.                                                        | No                 |
| `driver-gzip`          | Read Gzip compressed data.                                                           | No                 |
| `driver-h2`            | Connect to H2 through the embedded JDBC implementation.                              | No                 |
| `driver-http`          | Download and query data over HTTP.                                                   | No                 |
| `driver-https`         | Download and query data over HTTPS.                                                  | No                 |
| `driver-jdbc`          | Run JDBC drivers inside the embedded JVM.                                            | No                 |
| `driver-json`          | Query JSON files.                                                                    | No                 |
| `driver-jsonl`         | Query JSON Lines files.                                                              | No                 |
| `driver-lz4`           | Read LZ4 compressed data.                                                            | No                 |
| `driver-mariadb`       | Connect to MariaDB using `SQLx`.                                                     | No                 |
| `driver-mysql`         | Connect to MySQL using `SQLx`.                                                       | No                 |
| `driver-ods`           | Query `OpenDocument` Spreadsheet files.                                              | No                 |
| `driver-orc`           | Query ORC files.                                                                     | No                 |
| `driver-parquet`       | Query Parquet files.                                                                 | No                 |
| `driver-postgres`      | Connect to PostgreSQL using `tokio-postgres`, including managed local servers.       | No                 |
| `driver-postgresql`    | Connect to PostgreSQL using `SQLx`, including managed local servers.                 | No                 |
| `driver-redshift`      | Connect to Amazon Redshift through the PostgreSQL protocol.                          | No                 |
| `driver-rusqlite`      | Query SQLite databases using `rusqlite`.                                             | No                 |
| `driver-s3`            | Download and query data from S3.                                                     | No                 |
| `driver-scylladb`      | Connect to `ScyllaDB` using native CQL.                                              | No                 |
| `driver-snowflake`     | Connect to Snowflake through its SQL API.                                            | No                 |
| `driver-sqlite`        | Query SQLite databases using `SQLx`.                                                 | No                 |
| `driver-sqlserver`     | Connect to SQL Server using Tiberius.                                                | No                 |
| `driver-tsv`           | Query TSV files.                                                                     | No                 |
| `driver-xml`           | Query XML files.                                                                     | No                 |
| `driver-xz`            | Read XZ compressed data.                                                             | No                 |
| `driver-yaml`          | Query YAML files.                                                                    | No                 |
| `driver-zstd`          | Read Zstandard compressed data.                                                      | No                 |
| `format-ascii`         | Format results as tables with ASCII borders.                                         | No                 |
| `format-csv`           | Format results as comma-separated values.                                            | No                 |
| `format-expanded`      | Format each record vertically as field/value pairs.                                  | No                 |
| `format-html`          | Format results as HTML tables.                                                       | No                 |
| `format-json`          | Format results as JSON.                                                              | No                 |
| `format-jsonl`         | Format results as JSON Lines.                                                        | No                 |
| `format-markdown`      | Format results as Markdown tables.                                                   | No                 |
| `format-plain`         | Format results as text tables without borders.                                       | No                 |
| `format-psql`          | Format results as PostgreSQL `psql` tables.                                          | No                 |
| `format-sqlite`        | Format results as SQLite-style pipe-separated values.                                | No                 |
| `format-tsv`           | Format results as tab-separated values.                                              | No                 |
| `format-unicode`       | Format results as tables with Unicode borders.                                       | No                 |
| `format-xml`           | Format results as XML.                                                               | No                 |
| `format-yaml`          | Format results as YAML.                                                              | No                 |
| `tls-native-tls`       | Forward native TLS to enabled drivers; `ScyllaDB` uses its Rustls/AWS-LC alias.      | Yes                |
| `tls-rustls`           | Alias for selecting Rustls with the `ring` provider.                                 | No                 |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider in enabled drivers that expose this TLS feature. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider in enabled drivers that expose this TLS feature. | No                 |
