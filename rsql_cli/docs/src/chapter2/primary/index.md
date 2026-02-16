## primary

The `.primary` command displays primary key information for tables in your connected database. Primary keys
identify unique rows in a table and are critical for data integrity.

### Usage

```text
.primary [table]
```

### When to use

- Use `.primary` to list all primary keys in the database.
- Specify a table (e.g., `.primary users`) to see the primary key for that specific table.
- Use to understand how tables are uniquely identified, which is essential for joins and data integrity.

### Examples

Display the primary keys for all tables:

```text
.primary
```

Display the primary key for the `users` table:

```text
.primary users
```

### Output

The output includes:

- **Table** — the table name
- **Primary Key** — the constraint name
- **Columns** — the column(s) that make up the primary key
- **Inferred** — whether the primary key was declared in the schema or inferred from naming conventions
  (e.g., a NOT NULL column named `id`)

### Troubleshooting

- If no primary keys are shown, ensure your database supports primary key metadata and you have the
  necessary permissions.
- Some file-based or NoSQL data sources may not support primary keys natively; inferred keys may still
  be displayed.

### Related

- [describe](../describe/index.md) for full table structure
- [foreign](../foreign/index.md) for foreign keys
- [indexes](../indexes/index.md) for index information

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
