# `rsql_driver_sqlserver`

[![Documentation](https://docs.rs/rsql_driver_sqlserver/badge.svg)](https://docs.rs/rsql_driver_sqlserver)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_sqlserver.svg)](https://crates.io/crates/rsql_driver_sqlserver)
[![License](https://img.shields.io/crates/l/rsql_driver_sqlserver)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_sqlserver` connects rsql to Microsoft SQL Server using Tiberius.

## Usage

Driver URL format:

```text
sqlserver://[<user>[:<password>]@]<host>[:<port>][/<database>][?<options>]
```

```shell
rsql --url 'sqlserver://user:password@db.example.com:1433/example?encrypt=true&ApplicationName=rsql' -- 'SELECT @@VERSION;'
```

The default host is `localhost` and the default port is `1433`. Options are case-insensitive and
include `encrypt`, `TrustServerCertificateCA`, and `ApplicationName`. Windows builds also support
`IntegratedSecurity=true` for Windows integrated authentication.
