## output

The `.output` command redirects the output of rsql commands to a file or the system clipboard, instead of the default
stdout (console). This is useful for saving results, sharing data, or integrating with other tools.

### Usage

```text
.output [filename|clipboard]
```

### When to use

- Use `.output output.txt` to save results to a file for later analysis or sharing.
- Use `.output clipboard` to copy results directly to your system clipboard (supported platforms only).
- Use `.output` with no arguments to reset output to the console.

### Examples

Redirect output to the system clipboard:

```text
.output clipboard
```

Redirect output to a file named `output.txt`:

```text
.output output.txt
```

Reset output to the console:

```text
.output
```

### Troubleshooting

- If the clipboard option does not work, ensure your platform supports clipboard integration.
- If the file cannot be written, check file permissions and available disk space.

### Related

- For splitting output to multiple destinations, see [tee](../tee/index.md).
- For changing output format, see [format](../format/index.md).

