## timer

The `.timer` command toggles the display of the time taken to execute each query in rsql. This is useful for performance
monitoring, query optimization, and benchmarking different SQL statements or data sources.

### Usage

```text
.timer <on|off>
```

### When to use

- Enable `.timer on` to see how long each query takes—helpful for tuning queries or comparing database performance.
- Disable `.timer off` for a cleaner output if you do not need timing information.

### Examples

Show the current timer setting:

```text
.timer
```

Turn on the timer:

```text
.timer on
```

Turn off the timer:

```text
.timer off
```

### Troubleshooting

- If you do not see timing information, ensure `.timer on` is set.
- For minimal output, use `.timer off` in combination with `.header off`, `.footer off`, and `.rows off`.

### Related

- See the `timer` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For output customization, see [format](../format/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
