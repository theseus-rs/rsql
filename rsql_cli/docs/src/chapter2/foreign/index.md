## foreign

The `.foreign` command displays foreign key information for tables in your connected database. Foreign keys
define relationships between tables and are essential for understanding data models.

### Usage

```text
.foreign [table]
```

### When to use

- Use `.foreign` to list all foreign keys in the database.
- Specify a table (e.g., `.foreign orders`) to see foreign keys for that specific table.
- Use to understand table relationships, which is essential for writing joins and maintaining referential
  integrity.

### Examples

Display the foreign keys for all tables:

```text
.foreign
```

Display the foreign keys for the `orders` table:

```text
.foreign orders
```

### Output

The output includes:

- **Table** — the table name
- **Foreign Key** — the constraint name
- **Columns** — the local column(s) in the foreign key
- **Referenced Table** — the table being referenced
- **Referenced Columns** — the column(s) in the referenced table
- **Inferred** — whether the foreign key was declared in the schema or inferred from naming conventions
  (e.g., a `user_id` column referencing a `users` table with an `id` column)

### Troubleshooting

- If no foreign keys are shown, ensure your database supports foreign key metadata and you have the
  necessary permissions.
- Some file-based or NoSQL data sources may not support foreign keys natively; inferred keys may still
  be displayed.

### Related

- [describe](../describe/index.md) for full table structure
- [primary](../primary/index.md) for primary keys
- [indexes](../indexes/index.md) for index information

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
