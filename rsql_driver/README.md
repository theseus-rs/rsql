# `rsql_driver`

[![Documentation](https://docs.rs/rsql_driver/badge.svg)](https://docs.rs/rsql_driver)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver.svg)](https://crates.io/crates/rsql_driver)
[![License](https://img.shields.io/crates/l/rsql_driver)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver` is a library that provides a common interface for connecting to different data sources.

## Features

| Feature   | Description                                                 | Enabled by default |
|-----------|-------------------------------------------------------------|--------------------|
| `json`    | Convert `serde_json::Value` into the driver's `Value` type. | No                 |
