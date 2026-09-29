# `rsql_cli`

[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_cli.svg)](https://crates.io/crates/rsql_cli)
[![Github All Releases](https://img.shields.io/github/downloads/theseus-rs/rsql/total.svg)](https://theseus-rs.github.io/rsql/rsql_cli/)
[![License](https://img.shields.io/crates/l/rsql_cli)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql` is a command line interface for data.

## Getting Started

Install `rsql` from <https://theseus-rs.github.io/rsql/rsql_cli/>

<video width="640" height="480" controls>
  <source src="https://github.com/theseus-rs/rsql/blob/main/rsql_cli/resources/demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>

## Features

Default status follows this crate's `default` feature, including feature aliases.

| Feature                | Description                                                                          | Enabled by default |
|------------------------|--------------------------------------------------------------------------------------|--------------------|
| `repl`                 | Empty feature; the interactive REPL is always included.                              | Yes                |
| `tls-native-tls`       | Forward native TLS to enabled drivers                                                | Yes                |
| `tls-rustls`           | Alias for selecting Rustls with the `ring` provider.                                 | No                 |
| `tls-rustls-aws-lc-rs` | Use Rustls with the AWS-LC provider in enabled drivers that expose this TLS feature. | No                 |
| `tls-rustls-ring`      | Use Rustls with the `ring` provider in enabled drivers that expose this TLS feature. | No                 |
