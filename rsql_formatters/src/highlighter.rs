use crate::{FormatterOptions, Result};
use ansi_colours::ansi256_from_rgb;
use std::borrow::Cow;
use std::fmt::Write;
use supports_color::Stream;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::as_24_bit_terminal_escaped;

const RESET: &str = "\x1b[0m";

#[derive(Debug)]
pub struct Highlighter {
    color: bool,
    syntax_set: SyntaxSet,
    syntax: SyntaxReference,
    theme: Theme,
}

impl Highlighter {
    /// Create a new highlighter
    ///
    /// # Panics
    ///
    /// Panics if the syntax or theme cannot be found
    #[must_use]
    pub fn new(options: &FormatterOptions, syntax_name: &str) -> Self {
        let color = options.color;
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax = syntax_set
            .find_syntax_by_extension(syntax_name)
            .expect("syntax")
            .to_owned();
        let theme_set = ThemeSet::load_defaults();
        let theme_name = &options.theme;
        let theme = theme_set
            .themes
            .get(theme_name.as_str())
            .expect("theme")
            .to_owned();

        Self {
            color,
            syntax_set,
            syntax,
            theme,
        }
    }

    /// Highlight the content
    ///
    /// # Errors
    ///
    /// Returns an error if the content cannot be highlighted
    ///
    /// # Panics
    ///
    /// Panics if the content cannot be highlighted
    pub fn highlight<'l>(&self, content: &'l str) -> Result<Cow<'l, str>> {
        if !self.color {
            return Ok(content.into());
        }

        let Some(color_level) = supports_color::on(Stream::Stdout) else {
            return Ok(content.into());
        };

        let mut highlighter = HighlightLines::new(&self.syntax, &self.theme);
        let ranges: Vec<(Style, &str)> = highlighter
            .highlight_line(content, &self.syntax_set)
            .expect("highlight");

        if color_level.has_16m {
            return Ok((as_24_bit_terminal_escaped(&ranges[..], false) + RESET).into());
        } else if color_level.has_256 || color_level.has_basic {
            // Mac terminal.app - reports as 256 color support; works with 256 color
            // iTerm2 - reports as 256 color support; works with 24-bit color
            // Ubuntu Terminal - reports as has_basic; works with 24-bit color
            // Windows Terminal - reports as has_basic; works with 24-bit color
            return Ok(Self::as_256_color_terminal_escaped(&ranges));
        }

        // No color support
        Ok(content.into())
    }

    fn as_256_color_terminal_escaped<'l>(ranges: &[(Style, &'l str)]) -> Cow<'l, str> {
        let mut color_line: String = String::new();
        for &(ref style, text) in ranges {
            let foreground =
                ansi256_from_rgb([style.foreground.r, style.foreground.g, style.foreground.b]);
            write!(color_line, "\x1b[38;5;{foreground}m{text}").expect("write color");
        }

        write!(color_line, "{RESET}").expect("write reset");
        color_line.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_highlight_color_disabled() -> Result<()> {
        let options = FormatterOptions {
            color: false,
            ..Default::default()
        };
        let helper = Highlighter::new(&options, "sql");
        let line = "SELECT";
        let highlighted = helper.highlight(line)?;
        assert_eq!(highlighted, line);
        Ok(())
    }

    #[test]
    fn test_highlight_color_forced() -> Result<()> {
        let options = FormatterOptions {
            color: true,
            ..Default::default()
        };
        let helper = Highlighter::new(&options, "sql");
        let line = "SELECT";
        let highlighted = helper.highlight(line)?;
        assert!(highlighted.contains(line));
        Ok(())
    }
}
