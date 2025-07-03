## echo

The `.echo` command controls whether rsql echoes executed commands (and optionally the prompt) to the output. This is
useful for logging, debugging, or when you want to keep a record of all commands run in a session. By default, echo is
off.

### Usage

```text
.echo <on|prompt|off>
```

### When to use

- Enable echo (`on`) to log all commands for auditing or script debugging.
- Use `prompt` to echo both the prompt and commands, which is helpful for creating reproducible session logs.
- Disable echo (`off`) for a cleaner interactive experience.

### Examples

Show the current echo setting:

```text
.echo
```

Enable echoing commands:

```text
.echo on
```

Enable echoing the prompt and commands:

```text
.echo prompt
```

Disable echoing commands:

```text
.echo off
```

### Troubleshooting

- If your logs are missing commands, ensure `.echo on` or `.echo prompt` is set.
- For interactive use, keep echo off to avoid clutter.

### Related

- See the `echo` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For output redirection, see [output](../output/index.md) and [tee](../tee/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
