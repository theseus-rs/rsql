## views

The `.views` command lists all views available in the current schema or data source. This is useful for data
exploration, understanding database structure, and identifying available views for querying.

### Usage

```text
.views
```

### When to use

- Use `.views` to discover available views before writing queries.
- Helpful for exploring unfamiliar databases, verifying schema changes, or understanding the logical data model.

### Examples

List all views in the current schema:

```text
.views
```

### Troubleshooting

- If no views are listed, ensure you are connected to the correct schema and have the necessary permissions.
- Some data sources may require you to set the schema context first.
- Not all data sources support views. For example, file-based data sources and DynamoDB do not have views.

### Related

- [tables](../tables/index.md) for listing tables
- [schemas](../schemas/index.md) for schema selection
- [describe](../describe/index.md) for view or table details

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
