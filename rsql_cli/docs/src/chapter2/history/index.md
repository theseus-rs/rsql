## history

The `.history` command manages and displays the command history in rsql. This feature helps you recall, repeat, or edit
previous commands, improving productivity in interactive sessions. You can also enable or disable history tracking as
needed.

### Usage

```text
.history <on|off>
```

### When to use

- Use `.history` to review or search previous commands.
- Enable history (`on`) to keep a record of your session for future reference.
- Disable history (`off`) for privacy or when working with sensitive data.

### Examples

Show the current history setting and display the history:

```text
.history
```

Enable history tracking:

```text
.history on
```

Disable history tracking:

```text
.history off
```

### Troubleshooting

- If commands are not being saved, ensure `.history on` is set and your configuration allows history.
- For privacy, use `.history off` or clear the history file manually.

### Related

- See the `history.enabled` and `history.limit` options in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For command recall, use arrow keys or search shortcuts in your terminal.

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>

