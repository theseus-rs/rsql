# `rsql_driver_https`

[![Documentation](https://docs.rs/rsql_driver_http/badge.svg)](https://docs.rs/rsql_driver_https)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_https.svg)](https://crates.io/crates/rsql_driver_https)
[![License](https://img.shields.io/crates/l/rsql_driver_https)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_https` is a data http driver.

## Usage

Driver url format: `https://<path>[?_headers=<headers>]`

## Features

Default status follows this crate's `default` feature, including feature aliases.

| Feature                | Description                                             | Enabled by default |
|------------------------|---------------------------------------------------------|--------------------|
| `tls-native-tls`       | Use platform-native TLS for HTTPS requests.             | Yes                |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider for HTTPS requests. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider for HTTPS requests. | No                 |
