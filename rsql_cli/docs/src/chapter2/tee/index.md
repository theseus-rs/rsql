## tee

The `.tee` command duplicates the output of rsql commands to both the console (stdout) and a file or the system
clipboard. This is useful for logging, auditing, or sharing results while still seeing them interactively.

### Usage

```text
.tee [filename|clipboard]
```

### When to use

- Use `.tee output.txt` to save results to a file while also displaying them in the console.
- Use `.tee clipboard` to copy results to your clipboard and see them in the console (supported platforms only).
- Use `.tee` with no arguments to reset output to the console only.

### Examples

Redirect output to the system clipboard and the console:

```text
.tee clipboard
```

Redirect output to a file named `output.txt` and the console:

```text
.tee output.txt
```

Reset output to the console only:

```text
.tee
```

### Troubleshooting

- If the clipboard option does not work, ensure your platform supports clipboard integration.
- If the file cannot be written, check file permissions and available disk space.

### Related

- For output redirection without duplication, see [output](../output/index.md).
- For changing output format, see [format](../format/index.md).

