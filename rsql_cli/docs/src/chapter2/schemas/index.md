## schemas

The `.schemas` command lists all schemas available in the connected data source. Schemas are logical containers for
tables, views, and other database objects, and are especially important in multi-tenant or enterprise databases.

### Usage

```text
.schemas
```

### When to use

- Use `.schemas` to discover available schemas when connecting to complex databases.
- Helpful for exploring unfamiliar data sources, organizing queries, or verifying access permissions.

### Examples

List all schemas in the current data source:

```text
.schemas
```

### Troubleshooting

- If no schemas are listed, ensure your connection has the necessary permissions.
- Some databases may not support schemas; in that case, this command may return an empty result.

### Related

- See also: [catalogs](../catalogs/index.md) and [tables](../tables/index.md) for further exploration.

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
