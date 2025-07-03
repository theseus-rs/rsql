## bail

The `.bail` command controls how rsql handles errors during command execution. By default, rsql continues processing
after an error, which is useful for running scripts or multiple commands in a session. Enabling bail mode (`on`) will
cause rsql to immediately exit on the first error, which is helpful for automation, CI/CD, or when you want to ensure no
further actions are taken after a failure.

### Usage

```text
.bail <on|off>
```

### When to use

- **Enable bail** (`on`) when running scripts where any error should halt execution.
- **Disable bail** (`off`) for interactive sessions or when you want to review multiple errors in one run.

### Examples

Show the current bail setting:

```text
.bail
```

Enable bail on first error (recommended for automation):

```text
.bail on
```

Disable bail on first error (recommended for exploration):

```text
.bail off
```

### Troubleshooting

- If your script stops unexpectedly, check if `.bail on` is set.
- If errors are being ignored, ensure `.bail off` is not set unintentionally.

### Related

- See the `bail_on_error` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For error handling in scripts, see [Best Practices](../../chapter1/index.md#best-practices).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
