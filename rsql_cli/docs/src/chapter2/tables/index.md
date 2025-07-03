## tables

The `.tables` command lists all tables available in the current schema or data source. This is essential for data
exploration, query building, and verifying your access to specific tables.

### Usage

```text
.tables
```

### When to use

- Use `.tables` to discover available tables before writing queries.
- Helpful for exploring unfamiliar databases, verifying schema changes, or onboarding new users.

### Examples

List all tables in the current schema:

```text
.tables
```

### Troubleshooting

- If no tables are listed, ensure you are connected to the correct schema and have the necessary permissions.
- Some data sources may require you to set the schema context first.

### Related

- See also: [schemas](../schemas/index.md) for schema selection and [describe](../describe/index.md) for table details.

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
