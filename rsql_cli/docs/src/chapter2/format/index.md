## format

### Usage

```text
.format [format]
```

### Description

The format command sets the format mode for the CLI. The default format mode is `unicode`.

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

### Examples

Show the current format mode:

```text
.format
```

Set the format mode to `ascii`:

```text
.format ascii
```

Set the format mode to `unicode`:

```text
.format unicode
```

### Demonstration

<video width="640" height="480" controls>
  <source src="./demo.webm" type="video/webm">
  Your browser does not support the video tag.
</video>
