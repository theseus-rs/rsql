## catalogs

The `.catalogs` command lists all catalogs available in the connected data source. Catalogs are top-level containers for
schemas and tables, and are especially relevant in enterprise databases or cloud data warehouses.

### Usage

```text
.catalogs
```

### When to use

- Use `.catalogs` to discover available catalogs when connecting to complex or multi-tenant databases.
- Helpful for exploring unfamiliar data sources or verifying access permissions.

### Examples

List all catalogs in the current data source:

```text
.catalogs
```

### Troubleshooting

- If no catalogs are listed, ensure your connection has the necessary permissions.
- Some databases may not support catalogs; in that case, this command may return an empty result.

### Related

- See also: [schemas](../schemas/index.md) and [tables](../tables/index.md) commands for further exploration.

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
