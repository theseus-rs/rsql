# `rsql_driver_clickhouse`

[![Documentation](https://docs.rs/rsql_driver_clickhouse/badge.svg)](https://docs.rs/rsql_driver_clickhouse)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_clickhouse.svg)](https://crates.io/crates/rsql_driver_clickhouse)
[![License](https://img.shields.io/crates/l/rsql_driver_clickhouse)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_clickhouse` connects rsql to `ClickHouse` through its HTTP interface using the
`clickhouse` crate.

## Usage

Driver URL format:

```text
clickhouse://[<user>[:<password>]@]<host>[:<port>]/[<database>][?<options>]
```

```shell
rsql --url 'clickhouse://default@localhost:8123/default?scheme=http' -- 'SELECT version();'
```

The transport scheme defaults to `https`; set `scheme=http` for a plaintext endpoint. The default
host is `localhost` and the default port is `8123`, including when using HTTPS. Set the port to
match your service. The path selects the database, and `access_token` supplies an access token.
