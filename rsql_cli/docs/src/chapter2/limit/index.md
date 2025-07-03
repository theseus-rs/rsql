## limit

The `.limit` command sets the maximum number of rows displayed for query results in rsql. This is useful for controlling
output size, especially when working with large datasets or when you want to preview data without overwhelming your
terminal.

### Usage

```text
.limit [rows]
```

### When to use

- Use `.limit 10` to preview a small sample of your data.
- Use `.limit 0` to remove the row limit and display all results (be cautious with large tables).
- Adjust the limit for exporting, reporting, or interactive exploration.

### Examples

Display the current limit setting:

```text
.limit
```

Set row limit to unlimited (all rows):

```text
.limit 0
```

Set row limit to 10:

```text
.limit 10
```

### Troubleshooting

- If you see fewer rows than expected, check the current limit setting.
- For very large tables, avoid `.limit 0` unless you are sure your terminal and system can handle the output.

### Related

- See the `limit` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For output customization, see [format](../format/index.md), [header](../header/index.md),
  and [footer](../footer/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
