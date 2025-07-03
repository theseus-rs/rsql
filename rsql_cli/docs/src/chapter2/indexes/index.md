## indexes

The `.indexes` command displays index information for tables in your connected database. Indexes are critical for query
performance and data integrity, so this command is useful for database tuning, troubleshooting, and schema exploration.

### Usage

```text
.indexes [table]
```

### When to use

- Use `.indexes` to list all indexes in the database, helping you understand how queries are optimized.
- Specify a table (e.g., `.indexes users`) to see indexes relevant to that table, which is helpful for performance
  tuning or debugging slow queries.

### Examples

Display the indexes for all tables:

```text
.indexes
```

Display the indexes for the `users` table:

```text
.indexes users
```

### Troubleshooting

- If no indexes are shown, ensure your database supports index metadata and you have the necessary permissions.
- Some file-based or NoSQL data sources may not support indexes.

### Related

- See also: [describe](../describe/index.md) for table structure, [tables](../tables/index.md) for table listing,
  and [schemas](../schemas/index.md) for schema exploration.

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
