# `rsql_drivers`

[![Documentation](https://docs.rs/rsql_drivers/badge.svg)](https://docs.rs/rsql_drivers)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_drivers.svg)](https://crates.io/crates/rsql_drivers)
[![License](https://img.shields.io/crates/l/rsql_drivers)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_drivers` is a collection of SQL based data drivers crate.

## Features

Enable individual `driver-*` features to register the corresponding URL schemes. The default feature
set is empty. `all` includes every driver below; `all-wasm` includes the drivers marked Yes in the
last column.

WASM support depends on the host's filesystem and runtime facilities. Inclusion in `all-wasm` does
not imply that local files or network databases are accessible in a browser. JDBC and H2 require
preloaded JARs and Java runtime files on a host with filesystem access, such as WASI; Maven
downloads are native-only.

Each guide documents the URL format, connection options, defaults, and examples.

Default status follows this crate's `default` feature, including feature aliases.

| Feature                | Description                                                                                | Enabled by default | In `all-wasm`            |
|------------------------|--------------------------------------------------------------------------------------------|--------------------|--------------------------|
| `all`                  | Enable every driver; select a TLS backend separately.                                      | No                 | N/A                      |
| `all-wasm`             | Enable the driver subset for WebAssembly; see the WASM build limits below.                 | No                 | N/A                      |
| `driver-arrow`         | Query [Arrow IPC][arrow] files.                                                            | No                 | Yes                      |
| `driver-avro`          | Query [Avro][avro] files.                                                                  | No                 | Yes                      |
| `driver-brotli`        | Read [Brotli][brotli] compressed data.                                                     | No                 | Yes                      |
| `driver-bzip2`         | Read [Bzip2][bzip2] compressed data.                                                       | No                 | Yes                      |
| `driver-clickhouse`    | Connect to [`ClickHouse`][clickhouse] through its HTTP interface.                          | No                 | No                       |
| `driver-cockroachdb`   | Connect to [`CockroachDB`][cockroachdb] through the PostgreSQL protocol.                   | No                 | No                       |
| `driver-cratedb`       | Connect to [`CrateDB`][cratedb] through the PostgreSQL protocol.                           | No                 | No                       |
| `driver-csv`           | Query [CSV][csv] files.                                                                    | No                 | Yes                      |
| `driver-delimited`     | Query [Delimited text][delimited] files with a configurable delimiter.                     | No                 | Yes                      |
| `driver-duckdb`        | Query local or in-memory [`DuckDB`][duckdb] databases.                                     | No                 | No                       |
| `driver-dynamodb`      | Query [DynamoDB][dynamodb] through `PartiQL`.                                              | No                 | No                       |
| `driver-excel`         | Query [Excel][excel] workbooks.                                                            | No                 | Yes                      |
| `driver-file`          | Select a driver automatically using [File detection][file].                                | No                 | Yes                      |
| `driver-flightsql`     | Connect to [FlightSQL][flightsql] servers.                                                 | No                 | No                       |
| `driver-fwf`           | Query [Fixed-width text][fwf] files.                                                       | No                 | Yes                      |
| `driver-gzip`          | Read [Gzip][gzip] compressed data.                                                         | No                 | Yes                      |
| `driver-h2`            | Connect to [H2][h2] through the embedded JDBC implementation.                              | No                 | Yes (builds; see limits) |
| `driver-http`          | Download and query data over [HTTP][http].                                                 | No                 | No                       |
| `driver-https`         | Download and query data over [HTTPS][https].                                               | No                 | No                       |
| `driver-jdbc`          | Run [JDBC][jdbc] drivers inside the embedded JVM.                                          | No                 | Yes (builds; see limits) |
| `driver-json`          | Query [JSON][json] files.                                                                  | No                 | Yes                      |
| `driver-jsonl`         | Query [JSON Lines][jsonl] files.                                                           | No                 | Yes                      |
| `driver-lz4`           | Read [LZ4][lz4] compressed data.                                                           | No                 | Yes                      |
| `driver-mariadb`       | Connect to [MariaDB][mariadb] using `SQLx`.                                                | No                 | No                       |
| `driver-mysql`         | Connect to [MySQL][mysql] using `SQLx`.                                                    | No                 | No                       |
| `driver-ods`           | Query [`OpenDocument` Spreadsheet][ods] files.                                             | No                 | Yes                      |
| `driver-orc`           | Query [ORC][orc] files.                                                                    | No                 | Yes                      |
| `driver-parquet`       | Query [Parquet][parquet] files.                                                            | No                 | Yes                      |
| `driver-postgres`      | Connect to [PostgreSQL][postgres] using `tokio-postgres`, including managed local servers. | No                 | No                       |
| `driver-postgresql`    | Connect to [PostgreSQL][postgresql] using `SQLx`, including managed local servers.         | No                 | No                       |
| `driver-redshift`      | Connect to [Amazon Redshift][redshift] through the PostgreSQL protocol.                    | No                 | No                       |
| `driver-rusqlite`      | Query [SQLite][rusqlite] databases using `rusqlite`.                                       | No                 | No                       |
| `driver-s3`            | Download and query data from [S3][s3].                                                     | No                 | No                       |
| `driver-scylladb`      | Connect to [`ScyllaDB`][scylladb] using native CQL.                                        | No                 | No                       |
| `driver-snowflake`     | Connect to [Snowflake][snowflake] through its SQL API.                                     | No                 | No                       |
| `driver-sqlite`        | Query [SQLite][sqlite] databases using `SQLx`.                                             | No                 | No                       |
| `driver-sqlserver`     | Connect to [SQL Server][sqlserver] using Tiberius.                                         | No                 | No                       |
| `driver-tsv`           | Query [TSV][tsv] files.                                                                    | No                 | Yes                      |
| `driver-xml`           | Query [XML][xml] files.                                                                    | No                 | Yes                      |
| `driver-xz`            | Read [XZ][xz] compressed data.                                                             | No                 | Yes                      |
| `driver-yaml`          | Query [YAML][yaml] files.                                                                  | No                 | Yes                      |
| `driver-zstd`          | Read [Zstandard][zstd] compressed data.                                                    | No                 | Yes                      |
| `tls-native-tls`       | Forward native TLS to enabled drivers.                                                     | No                 | N/A                      |
| `tls-rustls`           | Alias for selecting Rustls with the `ring` provider.                                       | No                 | N/A                      |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider in enabled drivers that expose this TLS feature.       | No                 | N/A                      |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider in enabled drivers that expose this TLS feature.       | No                 | N/A                      |
| `tokio`                | Enable the optional Tokio dependency with filesystem support; registers no drivers.        | No                 | N/A                      |

[arrow]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/arrow.md
[avro]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/avro.md
[brotli]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/brotli.md
[bzip2]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/bzip2.md
[clickhouse]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/clickhouse.md
[cockroachdb]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/cockroachdb.md
[cratedb]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/cratedb.md
[csv]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/csv.md
[delimited]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/delimited.md
[duckdb]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/duckdb.md
[dynamodb]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/dynamodb.md
[excel]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/excel.md
[file]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/file.md
[flightsql]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/flightsql.md
[fwf]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/fwf.md
[gzip]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/gzip.md
[h2]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/h2.md
[http]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/http.md
[https]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/https.md
[jdbc]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/jdbc.md
[json]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/json.md
[jsonl]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/jsonl.md
[lz4]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/lz4.md
[mariadb]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/mariadb.md
[mysql]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/mysql.md
[ods]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/ods.md
[orc]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/orc.md
[parquet]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/parquet.md
[postgres]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/postgres.md
[postgresql]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/postgresql.md
[redshift]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/redshift.md
[rusqlite]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/rusqlite.md
[s3]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/s3.md
[scylladb]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/scylladb.md
[snowflake]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/snowflake.md
[sqlite]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/sqlite.md
[sqlserver]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/sqlserver.md
[tsv]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/tsv.md
[xml]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/xml.md
[xz]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/xz.md
[yaml]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/yaml.md
[zstd]: https://github.com/theseus-rs/rsql/blob/main/rsql_cli/docs/src/chapter3/zstd.md

## WASM build checks

JDBC and H2 can be checked independently for `wasm32-unknown-unknown` and `wasm32-wasip1`:

```shell
cargo check -p rsql_drivers --no-default-features \
  --features driver-h2,driver-jdbc --target wasm32-wasip1
```

The full `all-wasm` bundle currently encounters an upstream Polars/Tokio/Mio networking build
failure on `wasm32-unknown-unknown`. The `wasm32-wasip2` dependency graph also requires an unstable
`typed-path` feature with the current stable toolchain. These limits are separate from driver
registration and the JDBC/H2 checks above. A preview-1 H2 runtime smoke test also encounters
an unresolved upstream `wasi:io/streams@0.2.12` import with Wasmtime; compilation does not
guarantee execution on every WASI host.

## JDBC and H2 features

Both drivers are included in `all` and `all-wasm`. `driver-h2` uses the JDBC implementation
internally; enable `driver-jdbc` as well to register the generic `jdbc:` URL scheme.

On native targets, JDBC accepts repeatable `dependency=group:artifact:version` URL options and
resolves/caches their runtime dependencies through `ristretto_resolver`. H2 supplies its pinned
dependency automatically. Local `classpath` entries are also supported.

Select a TLS backend for native Maven and Java runtime downloads:

```shell
cargo build -p rsql_drivers --no-default-features \
  --features driver-h2,driver-jdbc,tls-rustls-ring
```

`tls-native-tls` and `tls-rustls-aws-lc-rs` are also supported. TLS features configure artifact
downloads; database connection security follows the JDBC driver's own settings. The standalone
driver crates default to `tls-rustls-ring`.

For WASM, use `--no-default-features`, supply JARs using `classpath` in the connection URL, and
make a preloaded Java runtime available to Ristretto through the host environment.
H2 omits its implicit Maven dependency on WASM. JDBC rejects explicit `dependency` options there
because its download/cache backend is native-only. See [JDBC WASM requirements][jdbc] and [H2 WASM
requirements][h2].
