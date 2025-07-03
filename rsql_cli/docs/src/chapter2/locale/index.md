## locale

The `.locale` command sets or displays the current locale used by rsql for formatting numbers, dates, and messages. This
is useful for international users or when you want to match output to a specific language or regional format.

### Usage

```text
.locale [locale]
```

### When to use

- Use `.locale` to check the current locale setting.
- Set a specific locale (e.g., `.locale en-GB`) to change language and number formatting.
- Useful for multi-language environments, demos, or when sharing results with users in different regions.

### Description

The locale command sets the locale for the CLI. The locale is used to display numeric values in the specified locale.
The default locale is determined by the system settings, or the `en` locale if the system settings can not be
determined.

### Examples

Show the current locale setting:

```text
.locale
```

Set the locale to British English:

```text
.locale en-GB
```

Set the locale to French:

```text
.locale fr
```

### Troubleshooting

- If you see untranslated or garbled text, ensure your locale is supported (
  see [Supported Locales](../../appendix/supported-locales.md)).
- Some output (such as database errors) may not be localized if not supported by the driver.

### Related

- See the `locale` option in [rsql.toml configuration](../../appendix/rsql-toml.md).
- For contributing translations, see [Supported Locales](../../appendix/supported-local
