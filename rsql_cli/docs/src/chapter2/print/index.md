## print

The `.print` command outputs a message to the current output destination (console, file, or clipboard if redirected).
This is useful for adding comments, separators, or debugging information in scripts and interactive sessions.

### Usage

```text
.print [string]
```

### When to use

- Use `.print` to display custom messages, progress updates, or script annotations.
- Helpful for marking sections in output files or logs.

### Examples

Print a message to the output:

```text
.print "hello, world!"
```

Print a separator line:

```text
.print "--------------------"
```

### Troubleshooting

- If you do not see the message, check if output is redirected (see `.output`).
- Ensure your string is properly quoted if it contains spaces or special characters.

### Related

- For output redirection, see [output](../output/index.md) and [tee](../tee/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
