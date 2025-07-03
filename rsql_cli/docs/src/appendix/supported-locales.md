## Supported Locales

Each locale includes a [ISO 639-1](https://www.iso.org/iso-639-language-code) language code and an
optional [ISO 3166-1](https://www.iso.org/iso-3166-country-codes.html) country code to specify language and regional
settings.

Locales affect the language of messages, prompts, and number formatting in rsql. You can set your preferred locale using
the `.locale` command or by specifying the `locale` option in your `rsql.toml` configuration file. If not set, rsql will
attempt to detect your system locale, defaulting to `en` (US English) if detection fails.

### Contributing Translations

Most translations are machine-generated and may be imperfect. To contribute improvements or add a new locale:

1. Fork the [rsql repository](https://github.com/theseus-rs/rsql).
2. Add or update the relevant translation files in the `locales/` directory.
3. Submit a pull request with your changes.
4. For guidance, see the project's contribution guidelines.

### Troubleshooting Locale Issues

- If you see untranslated or garbled text, ensure your locale is supported and correctly set.
- If your locale is not listed, contribute a translation as described above.
- Some output (such as database errors) may not be localized if not supported by the driver.

### Available Locales

| Locale | Description              |
|--------|--------------------------|
| ar     | Arabic                   |
| be     | Belarusian               |
| bg     | Bulgarian                |
| bn     | Bengali                  |
| cs     | Czech                    |
| da     | Danish                   |
| de     | German                   |
| el     | Greek                    |
| en-GB  | English (United Kingdom) |
| es     | Spanish                  |
| et     | Estonian                 |
| fi     | Finnish                  |
| fr     | French                   |
| ga     | Irish                    |
| he     | Hebrew                   |
| hi     | Hindi                    |
| hr     | Croatian                 |
| hu     | Hungarian                |
| is     | Icelandic                |
| it     | Italian                  |
| ja     | Japanese                 |
| jv     | Javanese                 |
| ka     | Georgian                 |
| ko     | Korean                   |
| lt     | Lithuanian               |
| lv     | Latvian                  |
| mk     | Macedonian               |
| ms     | Malay                    |
| mt     | Maltese                  |
| nl     | Dutch                    |
| no     | Norwegian                |
| pl     | Polish                   |
| pt     | Portuguese               |
| ro     | Romanian                 |
| ru     | Russian                  |
| sk     | Slovak                   |
| sl     | Slovenian                |
| sq     | Albanian                 |
| sr     | Serbian                  |
| sv     | Swedish                  |
| th     | Thai                     |
| tr     | Turkish                  |
| uk     | Ukrainian                |
| vi     | Vietnamese               |
| yi     | Yiddish                  |
| zh     | Chinese                  |
