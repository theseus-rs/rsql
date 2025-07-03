## describe

The `.describe` command provides detailed information about a table, including its columns, data types, and constraints.
This is useful for understanding the structure of your data, especially when working with unfamiliar tables or preparing
queries.

### Usage

```text
.describe [table]
```

### When to use

- Use `.describe` to inspect table schemas before writing queries or performing data transformations.
- Helpful for data exploration, debugging, and documentation.

### Examples

Describe the table named `users`:

```text
.describe users
```

Describe the current table (if context is set):

```text
.describe
```

### Troubleshooting

- If you receive an error, ensure the table name is correct and you have access permissions.
- Some data sources may require fully qualified table names (e.g., `schema.table`).

### Related

- See also: [tables](../tables/index.md), [schemas](../schemas/index.md), and [indexes](../indexes/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
