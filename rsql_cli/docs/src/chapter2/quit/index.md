## quit

The `.quit` command immediately exits the rsql CLI session. It is functionally equivalent to `.exit` but does not accept
an exit code. Use this command to leave the CLI at any time.

### Usage

```text
.quit
```

### When to use

- Use `.quit` to end your session quickly and cleanly.
- Useful for interactive sessions or when you want to ensure all resources are released.

### Examples

Exit the CLI:

```text
.quit
```

### Troubleshooting

- If the CLI does not exit, check for background operations or pending queries.
- For scripting or automation, prefer `.exit` if you need to specify an exit code.

### Related

- See also: [exit](../exit/index.md) for exiting with a status code.

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
