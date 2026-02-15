## describe

The `.describe` command provides detailed information about a table, including its columns, data types,
constraints, indexes, primary keys, and foreign key relationships. Primary keys and foreign keys are
displayed with an "Inferred" column that indicates whether the relationship was declared in the database
schema or inferred from column naming conventions (e.g., a `user_id` column referencing a `users` table,
or a NOT NULL `id` column as a primary key).

### Usage

```text
.describe [table]
```

### When to use

- Use `.describe` to inspect table schemas before writing queries or performing data transformations.
- Helpful for data exploration, debugging, and documentation.
- Use to discover primary key and foreign key relationships, including inferred ones based on naming conventions.

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

- [schemas](../schemas/index.md)
- [tables](../tables/index.md)
- [indexes](../indexes/index.md)
- [primary](../primary/index.md)
- [foreign](../foreign/index.md)

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
