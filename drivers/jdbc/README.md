# rsql_driver_jdbc

`rsql_driver_jdbc` runs JDBC drivers inside [Ristretto](https://github.com/theseus-rs/ristretto).
Ristretto runs the Java code in-process.

## Usage

Resolve the JDBC driver and its runtime dependencies from Maven Central. For example, connect to an
existing PostgreSQL database with an explicit driver class:

```sh
rsql --url 'jdbc:postgresql://localhost/example?user=postgres&dependency=org.postgresql:postgresql:42.7.13&driver=org.postgresql.Driver' -- 'SELECT version();'
```

The `driver` option names a class implementing `java.sql.Driver`. Omit it to use
`java.sql.DriverManager` and JDBC service discovery:

```sh
rsql --url 'jdbc:postgresql://localhost/example?user=postgres&dependency=org.postgresql:postgresql:42.7.13' -- 'SELECT 42;'
```

Repeat `dependency` for additional Maven artifacts, or supply local JARs with `classpath`. The [H2
driver](../h2/README.md) supplies its pinned Maven dependency and driver class through this same
JDBC implementation.

## URL options

Use a standard `jdbc:<subprotocol>:<database>` URL. These optional query parameters configure the
embedded JVM and are removed before the URL reaches JDBC:

| Parameter                       | Behavior                                                                                                                                                       |
|---------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `driver` (alias `driver_class`) | Instantiate this `java.sql.Driver` class and call its `connect` method. Without it, use `java.sql.DriverManager` and JDBC service discovery.                   |
| `dependency`                    | Maven `group:artifact:version` coordinate. May be repeated. Runtime dependencies are resolved together and cached; their JARs precede local classpath entries. |
| `classpath`                     | JARs/directories separated by the platform path separator (`:` on Unix, `;` on Windows). May be repeated. Defaults to `CLASSPATH`, then the current directory. |

Percent-encode option values containing URL delimiters, such as `&`, `?`, `+`, or spaces.
Database-specific query parameters retain their original encoding; JDBC semicolon settings also pass
through unchanged. JDBC driver options such as PostgreSQL's `user`, `password`, and `sslmode` are
passed to the selected driver. Use `&` to add JVM options when the JDBC URL already has a query
string.

Repeated `classpath` parameters work across platforms and avoid differences in path separators:

```sh
rsql --url 'jdbc:postgresql://localhost/example?user=postgres&classpath=/path/postgresql-42.7.13.jar&classpath=/path/dependency.jar&driver_class=org.postgresql.Driver'
```

Ristretto downloads and caches its default Java runtime libraries. The first connection needs
network access if those libraries are not already cached.

## Maven cache

`ristretto_resolver` stores Maven POMs, checksums, and JARs beneath the user's OS cache directory in
`rsql/maven/repository`. Resolved classpath artifacts are materialized in `rsql/maven/artifacts`.
Release entries are reused across sessions, including offline when all dependencies and runtime
libraries are already cached. Coordinates also accept `group:artifact:extension:version` and
`group:artifact:extension:classifier:version`.

## WebAssembly

The driver is included in `all-wasm`. WASM execution requires a host that provides filesystem
access, such as WASI, and preloaded JDBC JARs and Java runtime files. Supply the JARs using
`classpath` and make the runtime available to Ristretto through the host environment.
Automatic Maven downloads are native-only; `dependency` returns an explanatory error on WASM.
Browser-only `wasm32-unknown-unknown` builds compile, but cannot open
filesystem-backed JARs without host support. Database networking also depends on the host and JVM
APIs.

## Queries and metadata

The driver supports prepared parameters, affected-row counts, result labels, NULLs, numeric and
binary values, dates/times, UUIDs, arrays (including nested arrays), and JSON values. Prepared
statements use JDBC's `?` placeholders. Numeric values beyond `rust_decimal` precision and types
without an rsql value representation are returned as strings to preserve their text. Offset times
and timestamps retain their offsets as strings. Results are materialized in memory; statements and
result sets are closed on success and error. Call `close()` when finished with a connection.

Metadata uses JDBC's `DatabaseMetaData`: catalogs, schemas, and the current schema's tables, views,
columns, indexes, primary keys, and foreign keys. Transactions use the database's SQL syntax, such
as `BEGIN`, `COMMIT`, `ROLLBACK`, and savepoints in PostgreSQL. JDBC drivers run within Ristretto,
so compatibility also depends on the Java APIs that Ristretto supports.

## Cargo features

When using `rsql_drivers`, select `driver-jdbc` and a TLS backend for Java runtime downloads.
`rsql_repl` exposes the same driver feature. The standalone `rsql_driver_jdbc` crate defaults to
`tls-rustls-ring`; alternatives are `tls-native-tls` and `tls-rustls-aws-lc-rs` with default
features disabled.

These TLS features configure Maven and Ristretto runtime downloads. Database connection security is
configured through the JDBC driver's own URL options.
