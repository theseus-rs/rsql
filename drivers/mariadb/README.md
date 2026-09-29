# `rsql_driver_mariadb`

[![Documentation](https://docs.rs/rsql_driver_mariadb/badge.svg)](https://docs.rs/rsql_driver_mariadb)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_mariadb.svg)](https://crates.io/crates/rsql_driver_mariadb)
[![License](https://img.shields.io/crates/l/rsql_driver_mariadb)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_mariadb` connects rsql to MariaDB. It shares the `SQLx` implementation provided by
[rsql_driver_mysql](https://docs.rs/rsql_driver_mysql).

## Usage

Driver URL format:

```text
mariadb://<user>[:<password>]@<host>[:<port>]/<database>[?<options>]
```

```shell
rsql --url 'mariadb://user:password@localhost:3306/example' -- 'SELECT VERSION();'
```

The default TCP port is `3306`. Connection options include `ssl-mode`, `ssl-ca`, `ssl-cert`,
`ssl-key`, `charset`, and `socket`. Percent-encode reserved characters in credentials and option
values.

## Features

| Feature                | Description                                                         | Enabled by default |
|------------------------|---------------------------------------------------------------------|--------------------|
| `tls-native-tls`       | Use platform-native TLS for MySQL protocol connections.             | Yes                |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider for MySQL protocol connections. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider for MySQL protocol connections. | No                 |
