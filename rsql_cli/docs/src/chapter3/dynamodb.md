# DynamoDB

The `dynamodb` driver queries DynamoDB using PartiQL.

## URL format

```text
dynamodb://[<access_key_id>:<secret_access_key>@]<host>[:<port>][?<options>]
```

## Options

| Option          | Behavior / default                                                                                                             |
|-----------------|--------------------------------------------------------------------------------------------------------------------------------|
| `region`        | AWS region; defaults to the AWS SDK configuration chain.                                                                       |
| `session_token` | Session token accompanying explicit URL access-key credentials.                                                                |
| `scheme`        | Set `http` or `https` to use the URL host and port as a custom endpoint. Without it, the AWS SDK chooses the service endpoint. |

## Examples

```shell
rsql --url 'dynamodb://dynamodb.us-east-1.amazonaws.com?region=us-east-1'
rsql --url 'dynamodb://test:test@localhost:8000?scheme=http&region=us-east-1'
```

## Usage notes

Without URL credentials, the AWS SDK resolves credentials from its normal configuration chain. A
custom endpoint requires `scheme`; its default port is `443` when no port is given. The second
example connects to DynamoDB Local.

Use `.tables` to list tables, then query with PartiQL, for example `SELECT * FROM "people"`. PartiQL
support and affected-row reporting differ from relational SQL.
