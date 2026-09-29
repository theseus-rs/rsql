# `rsql_driver_h2`

`rsql_driver_h2` wraps the [JDBC driver](https://github.com/theseus-rs/rsql/blob/main/drivers/jdbc/README.md).

## Usage

Start a private in-memory database, or run a single query:

```sh
rsql --url 'h2://'
rsql --url 'h2://' -- 'SELECT H2VERSION();'
```

H2 supports named in-memory databases, persistent files, remote servers, and semicolon-separated
JDBC settings:

```sh
rsql --url 'h2:mem:example'
rsql --url 'h2:./example'
rsql --url 'h2:tcp://localhost/~/example'
rsql --url 'h2:mem:example;MODE=PostgreSQL'
```

`h2:` and `h2://` default to a private in-memory database (`jdbc:h2:mem:`). Other locations and
semicolon settings use H2's JDBC syntax. The optional `//` prefix is accepted for consistency with
other rsql drivers, so `h2://./example` and `h2:///absolute/path/example` work too. Connections
retain the original H2 URL in `Connection::url()`. A private in-memory database is discarded when
its connection closes; use a file URL such as `h2:./example` to retain data between sessions. Each
rsql JDBC connection owns a separate JVM, so named in-memory databases are not shared between
connections.

## Driver resolution and caching

Maven POMs, checksums, and JARs are cached beneath the user's OS cache directory at
`rsql/maven/repository`; the resolved JARs are materialized beneath `rsql/maven/artifacts`.
Immutable release entries are reused across sessions, including offline after the required artifacts
are cached. The resolver manages checksum validation and atomic downloads. No external Java process
is launched; Ristretto executes the JDBC driver inside rsql. The first connection needs network
access to fetch uncached Maven artifacts and Ristretto's default Java runtime libraries. Offline
use requires both the Maven artifacts and runtime libraries to be cached locally.

## URL options

The JDBC `dependency` and `classpath` options are also supported. Resolved H2 dependencies precede
additional classpath entries; the H2 driver class is always set implicitly to
`org.h2.Driver`, overriding any supplied `driver` or `driver_class`. Pass H2 connection settings
using semicolons, and JVM options using the query string:

```sh
rsql --url 'h2:./example;MODE=PostgreSQL?classpath=/path/to/extension.jar'
```

See the [JDBC options](https://github.com/theseus-rs/rsql/blob/main/drivers/jdbc/README.md#url-options) for encoding, classpath separators, and
environment-variable defaults. Query parameters, type conversion, and metadata use the [JDBC
implementation](https://github.com/theseus-rs/rsql/blob/main/drivers/jdbc/README.md#queries-and-metadata).

## Features

| Feature                | Description                                                               | Enabled by default |
|------------------------|---------------------------------------------------------------------------|--------------------|
| `tls-native-tls`       | Use platform-native TLS for Maven and Java runtime downloads.             | Yes                |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider for Maven and Java runtime downloads. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider for Maven and Java runtime downloads. | No                 |
