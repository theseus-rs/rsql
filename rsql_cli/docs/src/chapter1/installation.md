## Installation

To install rsql, use one of the commands below, or navigate to the [rsql](https://theseus-rs.github.io/rsql/rsql_cli/)
site and select an installation method. If you are attempting to install on a platform not
listed on the project site, you can find additional builds attached to
the [latest release](https://github.com/theseus-rs/rsql/releases/latest).

### Linux / MacOS

```shell
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/theseus-rs/rsql/releases/latest/download/rsql_cli-installer.sh | sh
```

### Windows

```shell
irm https://github.com/theseus-rs/rsql/releases/latest/download/rsql_cli-installer.ps1 | iex
```

### Troubleshooting Installation

- **Permission denied:** If you see a permission error, try running the installer with `sudo` (Linux/MacOS) or as
  Administrator (Windows).
- **Command not found:** Ensure the install directory is in your `PATH`. You may need to restart your terminal or add
  the install location to your shell profile.
- **Antivirus/Defender blocks installer:** Temporarily disable or whitelist the installer if you trust the source.
- **Unsupported platform:** Check the [latest release page](https://github.com/theseus-rs/rsql/releases/latest) for
  additional builds or open an issue for your platform.
- **Network issues:** If the installer fails to download, check your internet connection and proxy/firewall settings.

For more help, see the [FAQ](../appendix/index.md#faq--tips--tricks) or open an issue on
the [GitHub repository](https://github.com/theseus-rs/rsql/issues).

