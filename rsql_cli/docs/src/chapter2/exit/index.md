## exit

The `.exit` command immediately terminates the rsql session. You can optionally provide an exit code, which is useful
for scripting and automation to indicate success or failure to the calling process.

### Usage

```text
.exit [code]
```

### When to use

- Use `.exit` to leave the CLI at any time.
- Provide a non-zero exit code (e.g., `.exit 1`) to signal an error in scripts or CI/CD pipelines.

### Examples

Exit the shell with a status code of 0 (success):

```text
.exit
```

Exit the shell with a status code of 1 (failure):

```text
.exit 1
```

### Troubleshooting

- If the shell does not exit as expected, check for background processes or pending operations.
- In scripts, use `.exit <code>` to control flow based on success or failure.

### Related

- For quitting without specifying a code, see [quit](../quit/index.md).
- For error handling, see [bail](../bail/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
