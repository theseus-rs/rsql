use crate::shell::helper::ReplHelper;
use rustyline::highlight::Highlighter;
use std::borrow::Cow;

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line).expect("highlight")
    }

    fn highlight_char(&self, line: &str, pos: usize, _forced: bool) -> bool {
        let _ = (line, pos);
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use rustyline::ColorMode;

    #[test]
    fn test_highlight_color_disabled() {
        let configuration = Configuration {
            color_mode: ColorMode::Disabled,
            ..Default::default()
        };
        let helper = ReplHelper::new(&configuration);
        let line = "SELECT";
        let highlighted = helper.highlight(line, 0);
        assert!(highlighted.contains(line));
    }

    #[test]
    fn test_highlight_color_forced() {
        let configuration = Configuration {
            color_mode: ColorMode::Forced,
            ..Default::default()
        };
        let helper = ReplHelper::new(&configuration);
        let line = "SELECT";
        let highlighted = helper.highlight(line, 0);
        assert!(highlighted.contains(line));
    }

    #[test]
    fn test_highlight_char() {
        let configuration = Configuration::default();
        let helper = ReplHelper::new(&configuration);
        let line = "SELECT";
        let highlighted = helper.highlight_char(line, 0, false);
        assert!(highlighted);
    }
}
