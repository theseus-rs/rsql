# Drivers

Choose a driver using the scheme at the start of the connection URL. Every page below describes its
URL format, options, examples, and usage notes. Run `.drivers` to see which drivers are available in
your build.

```shell
rsql --url 'sqlite://' -- 'SELECT 42;'
rsql --url 'csv://people.csv' -- 'SELECT * FROM people LIMIT 10;'
```

Quote the URL in your shell, especially when it contains `?`, `&`, or semicolons. In the formats
below, angle brackets mark values to replace and square brackets mark optional parts; do not include
those brackets literally. Percent-encode reserved characters in credentials, filenames, and option
values.

File drivers accept relative and absolute paths. Most structured-file drivers load data into an
in-memory SQL context; database drivers connect directly to a database. Transport and compression
drivers delegate to a detected format driver. Their pages describe which options are forwarded.

## Databases and JDBC

| Driver        | Guide                                     |
|---------------|-------------------------------------------|
| `clickhouse`  | [ClickHouse](clickhouse.md)               |
| `cockroachdb` | [CockroachDB](cockroachdb.md)             |
| `cratedb`     | [CrateDB](cratedb.md)                     |
| `duckdb`      | [DuckDB](duckdb.md)                       |
| `dynamodb`    | [DynamoDB](dynamodb.md)                   |
| `flightsql`   | [FlightSQL](flightsql.md)                 |
| `h2`          | [H2](h2.md)                               |
| `jdbc`        | [JDBC](jdbc.md)                           |
| `mariadb`     | [MariaDB](mariadb.md)                     |
| `mysql`       | [MySQL](mysql.md)                         |
| `postgres`    | [PostgreSQL (rust-postgres)](postgres.md) |
| `postgresql`  | [PostgreSQL (SQLx)](postgresql.md)        |
| `redshift`    | [Amazon Redshift](redshift.md)            |
| `rusqlite`    | [SQLite (Rusqlite)](rusqlite.md)          |
| `scylladb`    | [ScyllaDB](scylladb.md)                   |
| `snowflake`   | [Snowflake](snowflake.md)                 |
| `sqlite`      | [SQLite (SQLx)](sqlite.md)                |
| `sqlserver`   | [SQL Server](sqlserver.md)                |

## Structured files

| Driver      | Guide                              |
|-------------|------------------------------------|
| `arrow`     | [Arrow IPC](arrow.md)              |
| `avro`      | [Avro](avro.md)                    |
| `csv`       | [CSV](csv.md)                      |
| `delimited` | [Delimited text](delimited.md)     |
| `excel`     | [Excel](excel.md)                  |
| `fwf`       | [Fixed-width text](fwf.md)         |
| `json`      | [JSON](json.md)                    |
| `jsonl`     | [JSON Lines](jsonl.md)             |
| `ods`       | [OpenDocument Spreadsheet](ods.md) |
| `orc`       | [ORC](orc.md)                      |
| `parquet`   | [Parquet](parquet.md)              |
| `tsv`       | [TSV](tsv.md)                      |
| `xml`       | [XML](xml.md)                      |
| `yaml`      | [YAML](yaml.md)                    |

## Files and remote resources

| Driver  | Guide                     |
|---------|---------------------------|
| `file`  | [File detection](file.md) |
| `http`  | [HTTP](http.md)           |
| `https` | [HTTPS](https.md)         |
| `s3`    | [S3](s3.md)               |

## Compression

| Driver   | Guide                |
|----------|----------------------|
| `brotli` | [Brotli](brotli.md)  |
| `bzip2`  | [Bzip2](bzip2.md)    |
| `gzip`   | [Gzip](gzip.md)      |
| `lz4`    | [LZ4](lz4.md)        |
| `xz`     | [XZ](xz.md)          |
| `zstd`   | [Zstandard](zstd.md) |
