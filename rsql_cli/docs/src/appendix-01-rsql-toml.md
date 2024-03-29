## Configuration File (rsql.toml)

A default [rsql.toml](#rsqltoml-1) file will be created on startup if one does not already exist.
This file is used to configure the behavior of the rsql CLI and will be written to the `$HOME/.rsql` directory
on Unix-like systems and `%APPDATA%\rsql` on Windows. The file is written in the [TOML](https://toml.io/en/) format.

### rsql.toml

The following is an example of a `rsql.toml` file:

```toml
[global]

# The locale to use.  If not specified, an attempt will be made to detect the system locale, but
# if that fails, the default "en" (US English) locale will be used.
#locale = "en"

# Indicate if the program should exit after the first error occurs.
#
# Possible values:
#   true - exit after the first error
#   false - continue processing after the first error
bail_on_error = false

# Indicate if color should be used in the output.
#
# Possible values:
#   true - use color in the output
#   false - don't use color in the output
#color = true

# The string used to initiate a command.
#
# This is used to determine if a line is a command or not. For example, if the command identifier
# is set to ".", then any line that starts with a "." will be considered a command.
command_identifier = "."

# Indicate if executed commands should be echoed to the defined output.
#
# Possible values:
#   true - echo executed commands
#   false - don't echo executed commands
echo = false

[log]

# The log level to use.
#
# Possible values:
#   "off"  - Designates that trace instrumentation should be completely disabled.
#   "error" - Designates very serious errors.
#   "warn" - Designates hazardous situations.
#   "info" - Designates useful information.
#   "debug" - Designates lower priority information.
#   "trace" - Designates very low priority, often extremely verbose, information.
level = "info"

# The frequency to rotate the logs.
#
# Possible values:
#   "minutely" - Rotate the logs minutely.
#   "hourly" - Rotate the logs hourly.
#   "daily" - Rotate the logs daily.
#   "never" - Never rotate the logs.
rotation = "daily"

[shell]

# The key binding mode to use.
#
# Possible values:
#   "emacs" - use the Emacs key bindings
#   "vi" - use the Vi key bindings
edit_mode = "emacs"

# Indicate if commands should be saved to the history file.
#
# Possible values:
#   true - save commands to the history file
#   false - don't save commands to the history file
history.enabled = true

# Indicate if duplicate commands should be saved to the history file.
#
# Possible values:
#   true - save duplicate commands to the history file
#   false - don't save duplicate commands to the history file
history.ignore_dups = true

# The maximum number of history entries to keep.
#
# 0 means no limit.
history.limit = 1000

# The theme to use when light mode is detected.
theme.light = "Solarized (light)"

# The theme to use when dark mode is detected.
theme.dark = "Solarized (dark)"

# The theme to use. This value overrides the light and dark mode themes when set.
#
# Possible values:
#   "base16-ocean.dark"
#   "base16-ocean.light"
#   "Solarized (dark)"
#   "Solarized (light)"
#theme = "Solarized (dark)"

[results]

# Indicate if changes should be displayed.
#
# Possible values:
#   true - display the changes
#   false - don't display the changes
changes = true

# The format to use for results.
#
# Possible values:
#   "ascii" - ASCII characters to draw a table
#   "csv" - Comma Separated Values (CSV)
#   "html" - HyperText Markup Language (HTML)
#   "json" - JavaScript Object Notation (JSON)
#   "jsonl" - JSON Lines (JSONL)
#   "markdown" - Markdown
#   "plain" - Column based layout
#   "psql" - PostgreSQL formatted table
#   "sqlite" - SQLite formatted table
#   "tsv" - Tab Separated Values (TSV)
#   "unicode" - Unicode characters to draw a table
#   "xml" - Extensible Markup Language (XML)
#   "yaml" - YAML Ain’t Markup Language (YAML)
format = "unicode"

# Indicate if header should be displayed when displaying results.
#
# Possible values:
#   true - display the header
#   false - don't display the header
header = true

# Indicate if footer should be displayed when displaying results.
#
# Possible values:
#   true - display the footer
#   false - don't display the footer
footer = true

# Enable timer for commands.
#
# Possible values:
#   true - enable timer
#   false - disable timer
timer = true
```
