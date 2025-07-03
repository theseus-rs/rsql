## footer

The `.footer` command controls whether rsql displays a footer after query results. The footer typically shows summary
information such as row counts, execution time, and other metadata. By default, the footer is displayed.

### Usage

```text
.footer <on|off>
```

### When to use

- Enable the footer (`on`) to see summary information after each query—useful for data analysis and performance
  monitoring.
- Disable the footer (`off`) for a cleaner output, especially when exporting results or scripting.

### Examples

Show the current footer setting:

```text
.footer
```

Enable the footer:

```text
.footer on
```

Disable the footer:

```text
.footer off
```

### Troubleshooting

- If you do not see summary information, ensure `.footer on` is set.
- For minimal output, use `.footer off` in combination with `.header off` and `.changes off`.

### Related

- See the `footer` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For output customization, see [format](../format/index.md), [header](../header/index.md),
  and [changes](../changes/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
