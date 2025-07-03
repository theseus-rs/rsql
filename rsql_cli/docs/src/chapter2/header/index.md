## header

The `.header` command controls whether rsql displays a header row (column names) above query results. By default, the
header is displayed, making it easier to interpret data, especially for wide tables or unfamiliar queries.

### Usage

```text
.header <on|off>
```

### When to use

- Enable the header (`on`) for interactive exploration, data analysis, or when sharing results with others.
- Disable the header (`off`) for minimal output, such as when exporting data for further processing.

### Examples

Show the current header setting:

```text
.header
```

Enable the header:

```text
.header on
```

Disable the header:

```text
.header off
```

### Troubleshooting

- If you do not see column names, ensure `.header on` is set.
- For scripting or exporting, use `.header off` to avoid extra lines in output files.

### Related

- See the `header` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For output customization, see [format](../format/index.md), [footer](../footer/index.md),
  and [changes](../changes/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
