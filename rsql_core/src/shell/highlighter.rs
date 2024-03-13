use crate::shell::helper::ReplHelper;
use ansi_colours::ansi256_from_rgb;
use rustyline::highlight::Highlighter;
use rustyline::ColorMode;
use std::borrow::Cow;
use std::fmt::Write;
use supports_color::Stream;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style;
use syntect::util::as_24_bit_terminal_escaped;

const RESET: &str = "\x1b[0m";

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if self.color_mode == ColorMode::Disabled {
            return line.into();
        }

        if let Some(support) = supports_color::on(Stream::Stdout) {
            let mut highlighter = HighlightLines::new(&self.syntax, &self.theme);
            let ranges: Vec<(Style, &str)> = highlighter
                .highlight_line(line, &self.syntax_set)
                .expect("highlight");

            if support.has_16m {
                return (as_24_bit_terminal_escaped(&ranges[..], false) + RESET).into();
            } else if support.has_256 || support.has_basic {
                // Mac terminal.app - reports as 256 color support; works with 256 color
                // iTerm2 - reports as 256 color support; works with 24-bit color
                // Ubuntu Terminal - reports as has_basic; works with 24-bit color
                // Windows Terminal - reports as has_basic; works with 24-bit color
                return as_256_color_terminal_escaped(&ranges);
            }
        }

        // No color support
        line.into()
    }

    fn highlight_char(&self, line: &str, pos: usize, _forced: bool) -> bool {
        let _ = (line, pos);
        true
    }
}

fn as_256_color_terminal_escaped<'l>(ranges: &[(Style, &'l str)]) -> Cow<'l, str> {
    let mut color_line: String = String::new();
    for &(ref style, text) in ranges.iter() {
        let foreground =
            ansi256_from_rgb([style.foreground.r, style.foreground.g, style.foreground.b]);
        write!(color_line, "\x1b[38;5;{}m{}", foreground, text).expect("write color");
    }

    write!(color_line, "{}", RESET).expect("write reset");
    color_line.into()
}
