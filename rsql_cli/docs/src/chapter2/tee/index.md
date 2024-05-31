## output

### Usage

```text
.tee [filename]
```

### Description

The tee command redirects the output of commands to the system clipboard or a file and stdout (console). If option is
provided, the output is redirected to stdout (console).

### Examples

Redirect the output of commands to the system clipboard and the console:

```text
.tee clipboard
```

Redirect the output of commands to a file named `output.txt` and the console:

```text
.tee output.txt
```

Redirect the output of commands to stdout (console):

```text
.tee
```
