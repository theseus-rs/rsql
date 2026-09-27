# JDBC

The `jdbc` driver runs Java JDBC drivers inside rsql using the embedded Ristretto JVM. It resolves
driver JARs and runtime dependencies from Maven Central, or loads local JARs supplied on a
classpath.

## URL format

```text
jdbc:<subprotocol>:<database>[?<database-options>&dependency=<coordinate>&driver=<class>]
```

Use the database's standard JDBC URL. For example, PostgreSQL uses
`jdbc:postgresql://<host>[:<port>]/<database>`. Database-specific query parameters and semicolon
settings pass through to the JDBC driver.

## Options

These options are removed before the URL reaches JDBC:

| Option                     | Behavior / default                                                                                                                                                                       |
|----------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `dependency`               | Maven coordinate `group:artifact:version`. May be repeated. Each artifact and its transitive runtime dependencies are resolved together and cached.                                      |
| `driver` or `driver_class` | Optional class implementing `java.sql.Driver`. Without it, `java.sql.DriverManager` uses JDBC service discovery.                                                                         |
| `classpath`                | Local JARs/directories, separated by `:` on Unix or `;` on Windows. May be repeated. Defaults to `CLASSPATH`; without any classpath or dependencies, the JVM uses the current directory. |

Maven coordinates also accept `group:artifact:extension:version` and
`group:artifact:extension:classifier:version`. Dependencies are resolved in URL order; resolved
artifacts precede local classpath entries. Use pinned release versions for repeatable connections.
Local classpaths remain usable without any `dependency` option.

Use `?` before the first query parameter and `&` between subsequent parameters. Percent-encode
delimiters in option values: `%20` for a space, `%2B` for a literal plus sign, and `%26` for an
ampersand. Database-specific options such as `user`, `password`, and `sslmode` keep their original
encoding.

## Examples

Resolve PostgreSQL JDBC and its runtime dependencies automatically:

```shell
rsql --url 'jdbc:postgresql://localhost/example?user=postgres&dependency=org.postgresql:postgresql:42.7.13&driver=org.postgresql.Driver' -- 'SELECT version();'
```

Use JDBC service discovery, or supply existing local JARs:

```shell
rsql --url 'jdbc:postgresql://localhost/example?user=postgres&dependency=org.postgresql:postgresql:42.7.13' -- 'SELECT 42;'
rsql --url 'jdbc:postgresql://localhost/example?user=postgres&classpath=/path/postgresql-42.7.13.jar&classpath=/path/dependency.jar&driver=org.postgresql.Driver'
```

Repeat `dependency` for additional published libraries. For H2, a generic JDBC connection can be
written as:

```shell
rsql --url 'jdbc:h2:mem:example?dependency=com.h2database:h2:2.5.252&driver=org.h2.Driver' -- 'SELECT H2VERSION();'
```

The dedicated [H2 driver](h2.md) supplies this dependency and driver class implicitly.

## Downloads and offline use

`ristretto_resolver` caches Maven POMs, checksums, and artifacts under the user's OS cache directory
in `rsql/maven/repository`. Resolved JARs are materialized in `rsql/maven/artifacts`. Cached release
entries are reused across connections; the resolver handles transitive dependency mediation and
checksum verification.

The first connection needs network access for uncached artifacts and Ristretto's default Java
runtime libraries. Offline use requires all needed Maven artifacts and runtime libraries to be
cached locally. No external Java process is launched.

## Usage notes

The driver supports affected-row counts, result labels, NULLs, numeric and binary values, dates and
times, UUIDs, arrays (including nested arrays), and JSON values. Numeric values exceeding rsql's
decimal precision are returned as strings. Times and timestamps with offsets retain their offsets
as strings. Results are collected in memory.
