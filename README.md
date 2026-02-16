<p align="center"><img width="250" height="250" src="rsql_cli/resources/rsql.png"></p>

# rsql

[![ci](https://github.com/theseus-rs/rsql/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/theseus-rs/rsql/actions/workflows/ci.yml)
[![Documentation](https://docs.rs/rsql_core/badge.svg)](https://docs.rs/rsql_core)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_cli.svg)](https://crates.io/crates/rsql_cli)
[![Github All Releases](https://img.shields.io/github/downloads/theseus-rs/rsql/total.svg)](https://theseus-rs.github.io/rsql/rsql_cli/)
[![License](https://img.shields.io/crates/l/rsql_cli)](https://github.com/theseus-rs/rsql_cli#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

> A modern, feature-rich command line SQL interface for data

`rsql` is a powerful and user-friendly command line SQL client that works with over 20 different data sources and
formats. Whether you're working with databases, files, or cloud services, `rsql` provides a consistent and intuitive
interface for all your data querying needs.

## Highlights

- **Universal SQL Interface**: Query databases, files, and cloud services with standard SQL
- **Rich Interactive Experience**: Syntax highlighting, auto-completion, and command history
- **Multiple Output Formats**: ASCII tables, JSON, CSV, HTML, and more
- **Extensive Data Source Support**: PostgreSQL, MySQL, SQLite, DuckDB, Parquet, CSV, Excel, and many more
- **Compression Support**: Automatically handles compressed files (gzip, brotli, etc.)
- **Embedded Database**: Run PostgreSQL queries without external setup
- **Multilingual**: Interface available in 40+ languages

[demo.webm](https://github.com/user-attachments/assets/613fdefb-753a-4bb2-acd2-66539d4f2068)

## Quick Start

### Installation

**Linux / MacOS:**

```shell
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/theseus-rs/rsql/releases/latest/download/rsql_cli-installer.sh | sh
```
or [Homebrew](https://brew.sh/):
```shell
brew install rsql
```

**Windows:**

```shell
irm https://github.com/theseus-rs/rsql/releases/latest/download/rsql_cli-installer.ps1 | iex
```

**Cargo**

```shell
cargo install rsql_cli
```

For detailed installation instructions, visit the [rsql documentation](https://theseus-rs.github.io/rsql/rsql_cli/).

### Basic Usage

**Interactive Mode:**

```shell
# Connect to a database
rsql --url "postgresql://user:pass@localhost/mydb"

# Query a CSV file
rsql --url "csv://data.csv"

# Use DuckDB for in-memory analytics
rsql --url "duckdb://"
```

**Execute Single Query:**

```shell
# Run a query and exit
rsql --url "sqlite://database.db" -- "SELECT * FROM users LIMIT 10"

# Query a Parquet file
rsql --url "parquet://data.parquet" -- "SELECT column1, COUNT(*) FROM table GROUP BY column1"
```

## Supported Data Sources

### Databases

- **ClickHouse** (`clickhouse://`)
- **CockroachDB** (`cockroachdb://`)
- **CrateDB** (`cratedb://`)
- **DuckDB** (`duckdb://`) - High-performance analytics
- **DynamoDB** (`dynamodb://`)
- **LibSQL/Turso** (`libsql://`)
- **MySQL** / **MariaDB** (`mysql://` / `mariadb://`)
- **PostgreSQL** (`postgresql://` / `postgres://`) - Including embedded PostgreSQL
- **Redshift** (`redshift://`)
- **Snowflake** (`snowflake://`)
- **SQL Server** (`sqlserver://`)
- **SQLite** (`sqlite://` / `rusqlite://`)

### File Formats

- **Arrow** (`arrow://`) - In-memory columnar format
- **Avro** (`avro://`) - Binary serialization format
- **CSV/TSV** (`csv://` / `tsv://`) - Comma/tab-separated values
- **Excel** (`excel://`) - .xlsx and .xls files
- **Fixed Width** (`fwf://`) - Fixed-width format
- **JSON/JSONL** (`json://` / `jsonl://`) - JSON documents
- **ODS** (`ods://`) - OpenDocument Spreadsheet
- **ORC** (`orc://`) - Optimized row columnar format
- **Parquet** (`parquet://`) - Columnar storage format
- **XML** (`xml://`) - XML documents
- **YAML** (`yaml://`) - YAML documents

### Cloud & Remote

- **FlightSQL** (`flightsql://`) - Apache Arrow Flight SQL
- **HTTP/HTTPS** (`http://` / `https://`) - Remote files
- **S3** (`s3://`) - Amazon S3 and S3-compatible storage

### Compression

Automatically handles: Gzip, Brotli, Bzip2, LZ4, XZ, Zstd

## Common Use Cases

### Database Operations

```shell
# Connect to PostgreSQL
rsql --url "postgresql://user:pass@localhost/mydb"

# Use embedded PostgreSQL (no external setup required)
rsql --url "postgresql://user@localhost/mydb?embedded=true"

# SQLite file
rsql --url "sqlite://database.db"
```

### File Analysis

```shell
# Analyze CSV data
rsql --url "csv://sales.csv" -- "SELECT region, SUM(revenue) FROM table GROUP BY region"

# Query Excel spreadsheet
rsql --url "excel://report.xlsx" -- "SELECT * FROM table WHERE amount > 1000"

# Analyze Parquet files
rsql --url "parquet://logs.parquet" -- "SELECT date, COUNT(*) FROM table GROUP BY date"
```

### Data Transformation

```shell
# Convert CSV to JSON
rsql --url "csv://input.csv" --format json -- "SELECT * FROM input"

# Query compressed files
rsql --url "csv://data.csv.gz" -- "SELECT column1, column2 FROM data"

# Combine multiple formats
rsql --url "duckdb://" -- "
  SELECT * FROM read_csv_auto('file1.csv') 
  UNION ALL 
  SELECT * FROM read_parquet('file2.parquet')
"
```

## Advanced Features

### Output Formats

Control output with `--format`:

- `ascii` - ASCII table (default)
- `csv` - Comma-separated values
- `html` - HTML table
- `json` - JSON format
- `markdown` - Markdown table
- `xml` - XML format
- `yaml` - YAML format
- And more...

### Configuration

```shell
# Set default format
rsql --url "sqlite://db.sqlite" --format json

# Custom delimiter for CSV output
rsql --url "postgresql://..." --format csv --delimiter ";"

# Expanded output for wide tables
rsql --url "..." --format expanded
```

## 📚 Examples

### File Format Examples

**CSV with custom options:**

```shell
rsql --url "csv://data.csv?has_header=true&skip_rows=1"
```

**Fixed-width file:**

```shell
rsql --url "fwf://data.txt?widths=10,20,15&headers=id,name,email"
```

**Delimited file with custom separator:**

```shell
rsql --url "delimited://data.txt?separator=|&has_header=true"
```

### Database Examples

**PostgreSQL with SSL:**

```shell
rsql --url "postgresql://user:pass@localhost/db?sslmode=require"
```

**MySQL with charset:**

```shell
rsql --url "mysql://user:pass@localhost/db?charset=utf8mb4"
```

## Features

| Feature               | Description                                                                                                                                                                                                                                  |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Data Sources          | Arrow, Avro, ClickHouse, CockroachDB, CrateDB, CSV, Delimited, DuckDB, DynamoDB, Excel, FlightSQL, FWF, JSON, JSONL, LibSQL (Turso), MariaDB, MySQL, ODS, ORC, Parquet, PostgreSQL, Redshift, Snowflake, SQLite3, SQL Server, TSV, XML, YAML |
| Compression           | Brotli, Bzip2, Gzip, LZ4, XZ, Zstd                                                                                                                                                                                                           |
| Syntax Highlighting   | ✅ Full SQL syntax highlighting                                                                                                                                                                                                               |
| Result Highlighting   | ✅ Color output for better readability                                                                                                                                                                                                        |
| Query Auto-completion | ✅ Smart completion for SQL keywords and table names                                                                                                                                                                                          |
| History               | ✅ Command history with search                                                                                                                                                                                                                |
| SQL File Execution    | ✅ Execute .sql files directly                                                                                                                                                                                                                |
| Embedded PostgreSQL   | ✅ No external PostgreSQL installation required                                                                                                                                                                                               |
| Output Formats        | ascii, csv, expanded, html, json, jsonl, markdown, plain, psql, sqlite, tsv, unicode, xml, yaml                                                                                                                                              |
| Localized Interface   | 40+ languages¹                                                                                                                                                                                                                               |
| Key Bindings          | emacs, vi                                                                                                                                                                                                                                    |

¹ Computer translations; human translations welcome

## 🔗 Connection URLs

| Driver             | URL                                                                                                                                                                              |
|--------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| arrow (polars)     | `arrow://<file>`                                                                                                                                                                 |
| avro (polars)      | `avro://<file>`                                                                                                                                                                  |
| brotli¹            | `brotli://<file>`                                                                                                                                                                |
| bzip2¹             | `bzip2://<file>`                                                                                                                                                                 |
| clickhouse         | `clickhouse://<user>[:<password>]@<host>[:<port>]/<database>[?access_token=<token>][&scheme=<http\|https>]`                                                                      |
| cockroachdb (sqlx) | `cockroachdb://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                   |
| cratedb (sqlx)     | `cratedb://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                       |
| csv (polars)       | `csv://<file>[?has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                                                                                                         |
| delimited (polars) | `delimited://<file>[?separator=<char>][&has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                                                                                |
| duckdb             | `duckdb://[<file>]`                                                                                                                                                              |
| dynamodb           | `dynamodb://[<access_key_id>:<secret_access_key>@]<host>[:<port>]>[?region=<region>][&session_token=<token>][&scheme=<http\|https>]`                                             |
| excel              | `excel://<file>[?has_header=<true\|false>][&skip_rows=<n>]`                                                                                                                      |
| file¹              | `file://<file>`                                                                                                                                                                  |
| flightsql          | `flightsql://<user[:password>]@<host>[:<port>][?scheme=<http\|https>]`                                                                                                           |
| fwf                | `fwf://<file>?widths=<widths>[&headers=<headers>]`                                                                                                                               |
| gzip¹              | `gzip://<file>`                                                                                                                                                                  |
| http¹              | `http://<path>[?_headers=<headers>]`                                                                                                                                             |
| https¹             | `https://<path>[?_headers=<headers>]`                                                                                                                                            |
| json (polars)      | `json://<file>`                                                                                                                                                                  |
| jsonl (polars)     | `jsonl://<file>`                                                                                                                                                                 |
| libsql²            | `libsql://<host>?[<memory=true>][&file=<database_file>][&auth_token=<token>]`                                                                                                    |
| lz4¹               | `lz4://<file>`                                                                                                                                                                   |
| mariadb (sqlx)     | `mariadb://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                       |
| mysql (sqlx)       | `mysql://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                         |
| ods                | `ods://<file>[?has_header=<true\|false>][&skip_rows=<n>]`                                                                                                                        |
| orc                | `orc://<file>`                                                                                                                                                                   |
| parquet (polars)   | `parquet://<file>`                                                                                                                                                               |
| postgres           | `postgres://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                                                                                      |
| postgresql (sqlx)  | `postgresql://<user>[:<password>]@<host>[:<port>]/<database>?<embedded=true>`                                                                                                    |
| redshift (sqlx)    | `redshift://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                      |
| rusqlite           | `rusqlite://[<file>]`                                                                                                                                                            |
| s3¹                | `s3://[<access_key_id>:<secret_access_key>@]<host>[:<port>]/<bucket>/<object>[?region=<region>][&session_token=<token>][&force_path_style=(true\|false)][&scheme=<http\|https>]` |
| snowflake          | `snowflake://<user>[:<token>]@<account>.snowflakecomputing.com/[?private_key_file=pkey_file&public_key_file=pubkey_file]`                                                        |
| sqlite (sqlx)      | `sqlite://[<file>]`                                                                                                                                                              |
| sqlserver          | `sqlserver://<user>[:<password>]@<host>[:<port>]/<database>`                                                                                                                     |
| tsv (polars)       | `tsv://<file>[?has_header=<true\|false>][&quote=<char>][&skip_rows=<n>]`                                                                                                         |
| xml                | `xml://<file>`                                                                                                                                                                   |
| xz¹                | `xz://<file>`                                                                                                                                                                    |
| yaml               | `yaml://<file>`                                                                                                                                                                  |
| zstd¹              | `zstd://<file>`                                                                                                                                                                  |

¹ the driver will attempt to detect the type of file and automatically use the appropriate driver.  
² `libsql` needs to be enabled with the `libsql` feature flag; it is disabled by default as it conflicts
with `rusqlite`.

## License

Licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

**Need help?** Check out the [documentation](https://theseus-rs.github.io/rsql/rsql_cli/)
or [open an issue](https://github.com/theseus-rs/rsql/issues) on GitHub.
