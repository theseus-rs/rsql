## echo

### Usage

```text
.echo <on|prompt|off>
```

### Description

Echo executed commands. By default, the echo command is set to `off`. If the `echo` command is set to `on`, the
commands will be echoed to the defined output. If the `echo` command is set to `prompt`, prompt and commands will
be echoed to the defined output.

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
