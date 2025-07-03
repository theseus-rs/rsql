## completions

The `.completions` command enables or disables smart command and SQL completions in rsql. Smart completions help you
write commands and queries faster by suggesting keywords, table names, and more as you type. By default, completions are
enabled.

### Usage

```text
.completions <on|off>
```

### When to use

- Enable completions (`on`) for interactive sessions to boost productivity and reduce typos.
- Disable completions (`off`) if you prefer manual entry or experience issues with suggestions.

### Examples

Show the current completions setting:

```text
.completions
```

Enable completions:

```text
.completions on
```

Disable completions:

```text
.completions off
```

### Troubleshooting

- If completions are not working, ensure `.completions on` is set and your terminal supports interactive input.
- Some drivers or remote sessions may limit available suggestions.

### Related

- See the `smart.completions` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For command history, see [history](../history/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
