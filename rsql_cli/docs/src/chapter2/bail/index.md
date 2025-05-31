## bail

### Usage

```text
.bail <on|off>
```

### Description

The bail command sets the behavior of the CLI when an error occurs. By default, the CLI will continue processing after
the first error. If the bail command is set to `on`, the CLI will exit after the first error.

### Examples

Show the current bail setting:

```text
.bail
```

Enable bail on first error:

```text
.bail on
```

Disable bail on first error:

```text
.bail off
```

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
