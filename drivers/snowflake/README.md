# `rsql_driver_snowflake`

[![Documentation](https://docs.rs/rsql_driver_snowflake/badge.svg)](https://docs.rs/rsql_driver_snowflake)
[![Code Coverage](https://codecov.io/gh/theseus-rs/rsql/branch/main/graph/badge.svg)](https://codecov.io/gh/theseus-rs/rsql)
[![Latest version](https://img.shields.io/crates/v/rsql_driver_snowflake.svg)](https://crates.io/crates/rsql_driver_snowflake)
[![License](https://img.shields.io/crates/l/rsql_driver_snowflake)](https://github.com/theseus-rs/rsql#license)
[![Semantic Versioning](https://img.shields.io/badge/%E2%9A%99%EF%B8%8F_SemVer-2.0.0-blue)](https://semver.org/spec/v2.0.0.html)

`rsql_driver_snowflake` connects rsql to Snowflake through the Snowflake SQL API over HTTPS.
It supports OAuth tokens and RSA key-pair authentication.

## Usage

Driver URL format:

```text
snowflake://<user>[:<oauth-token>]@<account>.snowflakecomputing.com/[?<options>]
```

```shell
rsql --url 'snowflake://user:oauth-token@account.snowflakecomputing.com/' -- 'SELECT CURRENT_VERSION();'
rsql --url 'snowflake://user@account.snowflakecomputing.com/?private_key_file=/path/private.pem&public_key_file=/path/public.pem'
```

The URL password field contains an OAuth bearer token. For key-pair authentication, omit the token
and supply both `private_key_file` and `public_key_file` as paths to PEM RSA key files. Select the
database, schema, warehouse, and role through SQL as supported by the service.
