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
| af     | Afrikaans                |
| am     | Amharic                  |
| ar     | Arabic                   |
| az     | Azerbaijani              |
| be     | Belarusian               |
| bg     | Bulgarian                |
| bn     | Bengali                  |
| bs     | Bosnian                  |
| ca     | Catalan                  |
| cs     | Czech                    |
| cy     | Welsh                    |
| da     | Danish                   |
| de     | German                   |
| el     | Greek                    |
| en     | English                  |
| en-GB  | English (United Kingdom) |
| eo     | Esperanto                |
| es     | Spanish                  |
| et     | Estonian                 |
| eu     | Basque                   |
| fa     | Persian                  |
| fi     | Finnish                  |
| fr     | French                   |
| fy     | Western Frisian          |
| ga     | Irish                    |
| gd     | Scottish Gaelic          |
| gl     | Galician                 |
| gu     | Gujarati                 |
| ha     | Hausa                    |
| he     | Hebrew                   |
| hi     | Hindi                    |
| hr     | Croatian                 |
| ht     | Haitian Creole           |
| hu     | Hungarian                |
| hy     | Armenian                 |
| id     | Indonesian               |
| ig     | Igbo                     |
| is     | Icelandic                |
| it     | Italian                  |
| ja     | Japanese                 |
| jv     | Javanese                 |
| ka     | Georgian                 |
| kk     | Kazakh                   |
| km     | Khmer                    |
| kn     | Kannada                  |
| ko     | Korean                   |
| ku     | Kurdish                  |
| ky     | Kyrgyz                   |
| la     | Latin                    |
| lb     | Luxembourgish            |
| lo     | Lao                      |
| lt     | Lithuanian               |
| lv     | Latvian                  |
| mg     | Malagasy                 |
| mi     | Maori                    |
| mk     | Macedonian               |
| ml     | Malayalam                |
| mn     | Mongolian                |
| mr     | Marathi                  |
| ms     | Malay                    |
| mt     | Maltese                  |
| my     | Burmese                  |
| ne     | Nepali                   |
| nl     | Dutch                    |
| no     | Norwegian                |
| ny     | Nyanja                   |
| or     | Odia                     |
| pa     | Punjabi                  |
| pl     | Polish                   |
| ps     | Pashto                   |
| pt     | Portuguese               |
| ro     | Romanian                 |
| ru     | Russian                  |
| rw     | Kinyarwanda              |
| sd     | Sindhi                   |
| si     | Sinhala                  |
| sk     | Slovak                   |
| sl     | Slovenian                |
| sm     | Samoan                   |
| sn     | Shona                    |
| so     | Somali                   |
| sq     | Albanian                 |
| sr     | Serbian                  |
| st     | Southern Sotho           |
| su     | Sundanese                |
| sv     | Swedish                  |
| sw     | Swahili                  |
| ta     | Tamil                    |
| te     | Telugu                   |
| tg     | Tajik                    |
| th     | Thai                     |
| tk     | Turkmen                  |
| tl     | Tagalog                  |
| tr     | Turkish                  |
| tt     | Tatar                    |
| ug     | Uyghur                   |
| uk     | Ukrainian                |
| ur     | Urdu                     |
| uz     | Uzbek                    |
| vi     | Vietnamese               |
| xh     | Xhosa                    |
| yi     | Yiddish                  |
| yo     | Yoruba                   |
| zh     | Chinese                  |
| zu     | Zulu                     |
