# `rsql_driver_jdbc`

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
driver](https://github.com/theseus-rs/rsql/blob/main/drivers/h2/README.md) supplies its pinned Maven dependency and driver class through this same
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

## Queries and metadata

The driver supports prepared parameters, affected-row counts, result labels, NULLs, numeric and
binary values, dates/times, UUIDs, arrays (including nested arrays), and JSON values. Prepared
statements use JDBC's `?` placeholders. Numeric values beyond `rust_decimal` precision and types
without an rsql value representation are returned as strings to preserve their text. Offset times
and timestamps retain their offsets as strings. Results are materialized in memory; statements and
result sets are closed on success and error. Call `close()` when finished with a connection.

## Features

| Feature                | Description                                                               | Enabled by default |
|------------------------|---------------------------------------------------------------------------|--------------------|
| `tls-native-tls`       | Use platform-native TLS for Maven and Java runtime downloads.             | Yes                |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider for Maven and Java runtime downloads. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider for Maven and Java runtime downloads. | No                 |
