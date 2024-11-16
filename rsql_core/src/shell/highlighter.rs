use crate::shell::helper::ReplHelper;
use rustyline::highlight::{CmdKind, Highlighter};
use std::borrow::Cow;

impl Highlighter for ReplHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line).expect("highlight")
    }

    fn highlight_char(&self, line: &str, pos: usize, _kind: CmdKind) -> bool {
        let _ = (line, pos);
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;

    #[test]
    fn test_highlight_color_disabled() {
        let configuration = Configuration {
            color: false,
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
            color: true,
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
        let highlighted = helper.highlight_char(line, 0, CmdKind::ForcedRefresh);
        assert!(highlighted);
    }
}
