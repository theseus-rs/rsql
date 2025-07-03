# Appendix

## FAQ & Tips & Tricks

### Frequently Asked Questions

**Q: rsql fails to connect to my database. What should I check?**

- Ensure your connection string (URL) is correct and credentials are valid.
- Check that the database server is running and accessible from your machine.
- Verify that the required driver is installed and supported (see [drivers](../chapter2/drivers/index.md)).
- For cloud databases, ensure your IP is whitelisted and network/firewall rules allow access.

**Q: How do I change the output format (CSV, JSON, etc.)?**

- Use the `.format` command or set the `format` option in your `rsql.toml` ([see configuration](./rsql-toml.md)).

**Q: How do I set or change my locale?**

- Use the `.locale` command (see [locale command](../chapter2/locale/index.md)) or set the `locale` option in your
  `rsql.toml`.

**Q: Where is the configuration file stored?**

- On Unix-like systems: `$HOME/.rsql/rsql.toml`
- On Windows: `%APPDATA%\rsql\rsql.toml`

**Q: How do I contribute a new translation or improve an existing one?**

- See [Supported Locales](./supported-locales.md#contributing-translations) for instructions.

### Tips & Tricks

- Use command history and smart completions to speed up repetitive tasks.
- Use the `.read` command to execute SQL from a file.
- Use the `.output` command to redirect results to a file.
- For large result sets, use the `limit` option or `.limit` command to avoid overwhelming your terminal.
- Use the `.help` command to see available commands and their usage.

---

For more troubleshooting and advanced usage, see the [Configuration File](./rsql-toml.md)
and [Commands](../chapter2/index.md) sections.

