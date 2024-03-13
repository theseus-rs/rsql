## locale

### Usage

```text
.locale [locale]
```

### Description

The locale command sets the locale for the CLI. The locale is used to display numeric values in the specified locale.
The default locale is determined by the system settings, or the `en` locale if the system settings can not be
determined.

### Examples

Show the current locale setting:

```text
.locale
```

Set the locale to `en-GB`:

```text
.locale en-GB
```
