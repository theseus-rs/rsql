# `rsql_formatters`

[![Documentation](https://docs.rs/rsql_formatters/badge.svg)](https://docs.rs/rsql_formatters)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_formatters.svg)](https://crates.io/crates/rsql_formatters)
[![License](https://img.shields.io/crates/l/rsql_formatters)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_formatters` is a collection of formatters for the `rsql_drivers` crate.

## Features

| Feature    | Description                                              | Enabled by default |
|------------|----------------------------------------------------------|--------------------|
| `all`      | Enable every output format.                              | No                 |
| `ascii`    | Format results as tables with ASCII borders.             | No                 |
| `csv`      | Format results as comma-separated values.                | No                 |
| `expanded` | Format each record vertically as field/value pairs.      | No                 |
| `html`     | Format results as HTML tables.                           | No                 |
| `json`     | Format results as JSON.                                  | No                 |
| `jsonl`    | Format results as JSON Lines.                            | No                 |
| `markdown` | Format results as Markdown tables.                       | No                 |
| `plain`    | Format results as text tables without borders.           | No                 |
| `psql`     | Format results as PostgreSQL `psql` tables.              | No                 |
| `sqlite`   | Format results as SQLite-style pipe-separated values.    | No                 |
| `tsv`      | Format results as tab-separated values.                  | No                 |
| `unicode`  | Format results as tables with Unicode borders.           | No                 |
| `xml`      | Format results as XML.                                   | No                 |
| `yaml`     | Format results as YAML.                                  | No                 |
