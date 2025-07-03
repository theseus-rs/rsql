## format

The `.format` command sets the output format for query results in rsql. This allows you to tailor the display for
readability, data export, or integration with other tools. The default format is `psql`, but many formats are available
for different use cases.

### Usage

```text
.format [format]
```

### Available Formats

| Format     | Description                                                                         |
|------------|-------------------------------------------------------------------------------------|
| `ascii`    | ASCII characters to draw a table                                                    |
| `csv`      | [Comma Separated Values (CSV)](https://www.ietf.org/rfc/rfc4180.txt)                |
| `expanded` | [PostgreSQL Expanded Format](https://www.postgresql.org/docs/current/app-psql.html) |
| `html`     | [HyperText Markup Language (HTML)](https://html.spec.whatwg.org/multipage/)         |
| `json`     | [JavaScript Object Notation (JSON)](https://datatracker.ietf.org/doc/html/rfc8259)  |
| `jsonl`    | [JSON Lines (JSONL)](https://jsonlines.org/)                                        |
| `markdown` | [Markdown](https://www.markdownguide.org/extended-syntax/#tables)                   |
| `plain`    | Column based layout                                                                 |
| `psql`     | [PostgreSQL Standard Format](https://www.postgresql.org/docs/current/app-psql.html) |
| `sqlite`   | SQLite formatted table                                                              |
| `tsv`      | [Tab Separated Values (TSV)](https://en.wikipedia.org/wiki/Tab-separated_values)    |
| `unicode`  | Unicode characters to draw a table                                                  |
| `xml`      | [Extensible Markup Language (XML)](https://www.w3.org/TR/xml11/)                    |
| `yaml`     | [YAML Ain’t Markup Language (YAML)](https://yaml.org/spec/1.2.2/)                   |

### When to use

- Use `unicode`, `ascii`, or `psql` for interactive exploration.
- Use `csv`, `tsv`, `json`, `jsonl`, `xml`, or `yaml` for exporting data or integrating with other tools.
- Use `markdown` or `html` for documentation or reporting.
- Use `expanded` for wide tables or when you want each row displayed vertically.

### Examples

Show the current format mode:

```text
.format
```

Set the format mode to `ascii`:

```text
.format ascii
```

Set the format mode to `json` for machine-readable output:

```text
.format json
```

### Troubleshooting

- If output looks garbled in your terminal, try switching to `ascii` or `plain`.
- For large exports, prefer `csv`, `tsv`, or `jsonl` for best performance.

### Related

- See the `format` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For header/footer control, see [header](../header/index.md) and [footer](../footer/index.md).

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
