# rsql_drivers

[![Documentation](https://docs.rs/rsql_drivers/badge.svg)](https://docs.rs/rsql_drivers)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_drivers.svg)](https://crates.io/crates/rsql_drivers)
[![License](https://img.shields.io/crates/l/rsql_drivers)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_drivers` is a collection of SQL based data drivers crate.

## Driver features

Enable individual `driver-*` features to register the corresponding URL schemes. The default feature
set is empty. `all` includes every driver below; `all-wasm` includes the drivers marked Yes in the
WASM column.

WASM support depends on the host's filesystem and runtime facilities. Inclusion in `all-wasm` does
not imply that local files or network databases are accessible in a browser. JDBC and H2 require
preloaded JARs and Java runtime files on a host with filesystem access, such as WASI; Maven
downloads are native-only.

Each guide documents the URL format, connection options, defaults, and examples.

| Feature | Driver guide | WASM |
| --- | --- | --- |
| `driver-arrow` | [Arrow IPC][arrow] | Yes |
| `driver-avro` | [Avro][avro] | Yes |
| `driver-brotli` | [Brotli][brotli] | Yes |
| `driver-bzip2` | [Bzip2][bzip2] | Yes |
| `driver-clickhouse` | [ClickHouse][clickhouse] | No |
| `driver-cockroachdb` | [CockroachDB][cockroachdb] | No |
| `driver-cratedb` | [CrateDB][cratedb] | No |
| `driver-csv` | [CSV][csv] | Yes |
| `driver-delimited` | [Delimited text][delimited] | Yes |
| `driver-duckdb` | [DuckDB][duckdb] | No |
| `driver-dynamodb` | [DynamoDB][dynamodb] | No |
| `driver-excel` | [Excel][excel] | Yes |
| `driver-file` | [File detection][file] | Yes |
| `driver-flightsql` | [FlightSQL][flightsql] | No |
| `driver-fwf` | [Fixed-width text][fwf] | Yes |
| `driver-gzip` | [Gzip][gzip] | Yes |
| `driver-h2` | [H2][h2] | Yes (builds; see limits) |
| `driver-http` | [HTTP][http] | No |
| `driver-https` | [HTTPS][https] | No |
| `driver-jdbc` | [JDBC][jdbc] | Yes (builds; see limits) |
| `driver-json` | [JSON][json] | Yes |
| `driver-jsonl` | [JSON Lines][jsonl] | Yes |
| `driver-lz4` | [LZ4][lz4] | Yes |
| `driver-mariadb` | [MariaDB][mariadb] | No |
| `driver-mysql` | [MySQL][mysql] | No |
| `driver-ods` | [OpenDocument Spreadsheet][ods] | Yes |
| `driver-orc` | [ORC][orc] | Yes |
| `driver-parquet` | [Parquet][parquet] | Yes |
| `driver-postgres` | [PostgreSQL (rust-postgres)][postgres] | No |
| `driver-postgresql` | [PostgreSQL (SQLx)][postgresql] | No |
| `driver-redshift` | [Amazon Redshift][redshift] | No |
| `driver-rusqlite` | [SQLite (Rusqlite)][rusqlite] | No |
| `driver-s3` | [S3][s3] | No |
| `driver-scylladb` | [ScyllaDB][scylladb] | No |
| `driver-snowflake` | [Snowflake][snowflake] | No |
| `driver-sqlite` | [SQLite (SQLx)][sqlite] | No |
| `driver-sqlserver` | [SQL Server][sqlserver] | No |
| `driver-tsv` | [TSV][tsv] | Yes |
| `driver-xml` | [XML][xml] | Yes |
| `driver-xz` | [XZ][xz] | Yes |
| `driver-yaml` | [YAML][yaml] | Yes |
| `driver-zstd` | [Zstandard][zstd] | Yes |

[arrow]: ../rsql_cli/docs/src/chapter3/arrow.md
[avro]: ../rsql_cli/docs/src/chapter3/avro.md
[brotli]: ../rsql_cli/docs/src/chapter3/brotli.md
[bzip2]: ../rsql_cli/docs/src/chapter3/bzip2.md
[clickhouse]: ../rsql_cli/docs/src/chapter3/clickhouse.md
[cockroachdb]: ../rsql_cli/docs/src/chapter3/cockroachdb.md
[cratedb]: ../rsql_cli/docs/src/chapter3/cratedb.md
[csv]: ../rsql_cli/docs/src/chapter3/csv.md
[delimited]: ../rsql_cli/docs/src/chapter3/delimited.md
[duckdb]: ../rsql_cli/docs/src/chapter3/duckdb.md
[dynamodb]: ../rsql_cli/docs/src/chapter3/dynamodb.md
[excel]: ../rsql_cli/docs/src/chapter3/excel.md
[file]: ../rsql_cli/docs/src/chapter3/file.md
[flightsql]: ../rsql_cli/docs/src/chapter3/flightsql.md
[fwf]: ../rsql_cli/docs/src/chapter3/fwf.md
[gzip]: ../rsql_cli/docs/src/chapter3/gzip.md
[h2]: ../rsql_cli/docs/src/chapter3/h2.md
[http]: ../rsql_cli/docs/src/chapter3/http.md
[https]: ../rsql_cli/docs/src/chapter3/https.md
[jdbc]: ../rsql_cli/docs/src/chapter3/jdbc.md
[json]: ../rsql_cli/docs/src/chapter3/json.md
[jsonl]: ../rsql_cli/docs/src/chapter3/jsonl.md
[lz4]: ../rsql_cli/docs/src/chapter3/lz4.md
[mariadb]: ../rsql_cli/docs/src/chapter3/mariadb.md
[mysql]: ../rsql_cli/docs/src/chapter3/mysql.md
[ods]: ../rsql_cli/docs/src/chapter3/ods.md
[orc]: ../rsql_cli/docs/src/chapter3/orc.md
[parquet]: ../rsql_cli/docs/src/chapter3/parquet.md
[postgres]: ../rsql_cli/docs/src/chapter3/postgres.md
[postgresql]: ../rsql_cli/docs/src/chapter3/postgresql.md
[redshift]: ../rsql_cli/docs/src/chapter3/redshift.md
[rusqlite]: ../rsql_cli/docs/src/chapter3/rusqlite.md
[s3]: ../rsql_cli/docs/src/chapter3/s3.md
[scylladb]: ../rsql_cli/docs/src/chapter3/scylladb.md
[snowflake]: ../rsql_cli/docs/src/chapter3/snowflake.md
[sqlite]: ../rsql_cli/docs/src/chapter3/sqlite.md
[sqlserver]: ../rsql_cli/docs/src/chapter3/sqlserver.md
[tsv]: ../rsql_cli/docs/src/chapter3/tsv.md
[xml]: ../rsql_cli/docs/src/chapter3/xml.md
[xz]: ../rsql_cli/docs/src/chapter3/xz.md
[yaml]: ../rsql_cli/docs/src/chapter3/yaml.md
[zstd]: ../rsql_cli/docs/src/chapter3/zstd.md

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

## Safety

These crates use `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](../LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
