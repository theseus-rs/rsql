# Snowflake

The `snowflake` driver sends SQL through the Snowflake SQL API.

## URL format

```text
snowflake://<user>[:<oauth-token>]@<account>.snowflakecomputing.com/[?<options>]
```

## Options

| Option             | Behavior / default                                                  |
|--------------------|---------------------------------------------------------------------|
| `private_key_file` | PEM RSA private-key file; required when no OAuth token is supplied. |
| `public_key_file`  | PEM RSA public-key file; required with `private_key_file`.          |

## Examples

```shell
rsql --url 'snowflake://user:oauth-token@account.snowflakecomputing.com/' -- 'SELECT CURRENT_VERSION();'
rsql --url 'snowflake://user@account.snowflakecomputing.com/?private_key_file=/path/private.pem&public_key_file=/path/public.pem'
```

## Usage notes

The password portion of the URL is an OAuth bearer token, not a Snowflake account password. With no
token, both key files are required for key-pair authentication. The connection uses HTTPS. Select
database, schema, warehouse, and role through SQL as supported by the service; these are not rsql
URL options.
