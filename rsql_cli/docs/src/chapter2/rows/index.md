## rows

The `.rows` command toggles the display of the number of rows returned by a query in rsql. This is useful for quickly
verifying the size of your result set, especially when filtering or aggregating data.

### Usage

```text
.rows <on|off>
```

### When to use

- Enable `.rows on` to always see how many rows your queries return—helpful for data validation and exploration.
- Disable `.rows off` for a cleaner output, especially when exporting results or scripting.

### Examples

Show the current rows setting:

```text
.rows
```

Turn on the rows returned display:

```text
.rows on
```

Turn off the rows returned display:

```text
.rows off
```

### Troubleshooting

- If you do not see row counts, ensure `.rows on` is set.
- For minimal output, use `.rows off` in combination with `.header off` and `.footer off`.

### Related

- See the `rows` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For output customization, see [format](../format/index.md), [header](../header/index.md),
  and [footer](../footer/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
