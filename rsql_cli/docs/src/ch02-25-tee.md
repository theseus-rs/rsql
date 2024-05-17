## output

### Usage

```text
.tee [filename]
```

### Description

The tee command redirects the output of SQL commands to a file and stdout (console). If no filename is provided, the
output is redirected to stdout (console).

### Examples

Redirect the output of SQL commands to a file named `output.txt` and the console:

```text
.tee output.txt
```

Redirect the output of SQL commands to the stdout (console):

```text
.tee
```
