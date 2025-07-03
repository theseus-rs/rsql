## changes

The `.changes` command toggles the display of the number of rows affected by SQL statements (such as INSERT, UPDATE,
DELETE). This feedback is useful for verifying the impact of your queries, especially in data modification or ETL
workflows.

### Usage

```text
.changes <on|off>
```

### When to use

- Enable `.changes on` to always see how many rows were changed by your queries—helpful for auditing and debugging.
- Disable `.changes off` for a cleaner output if you do not need this information.

### Examples

Show the current changes setting:

```text
.changes
```

Turn on the rows changed display:

```text
.changes on
```

Turn off the rows changed display:

```text
.changes off
```

### Troubleshooting

- If you do not see row change counts, ensure `.changes on` is set.
- Some drivers or read-only queries may not report row changes.

### Related

- See the `changes` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For output customization, see [format](../format/index.md) and [footer](../footer/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
