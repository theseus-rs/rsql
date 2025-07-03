## color

The `.color` command controls whether rsql outputs colored text in the terminal. Color output improves readability,
especially for large result sets or when distinguishing between different types of output. By default, color is enabled.

### Usage

```text
.color <on|off>
```

### When to use

- Enable color (`on`) for interactive use, demos, or when you want visually distinct output.
- Disable color (`off`) for scripts, logs, or when redirecting output to files where ANSI codes are undesirable.

### Examples

Show the current color setting:

```text
.color
```

Enable color output:

```text
.color on
```

Disable color output:

```text
.color off
```

### Troubleshooting

- If you see strange characters in redirected output, try `.color off`.
- Some terminals may not support color; in that case, disabling color is recommended.

### Related

- See the `color` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For output customization, see [format](../format/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
