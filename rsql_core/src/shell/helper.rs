use crate::configuration::Configuration;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{ColorMode, Context, Helper};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

pub(crate) struct ReplHelper {
    pub(crate) color_mode: ColorMode,
    pub(crate) syntax_set: SyntaxSet,
    pub(crate) syntax: SyntaxReference,
    pub(crate) theme: Theme,
}

impl ReplHelper {
    pub(crate) fn new(configuration: &Configuration) -> Self {
        let color_mode = configuration.color_mode;
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let syntax = syntax_set
            .find_syntax_by_extension("sql")
            .expect("sql syntax")
            .to_owned();
        let theme_set = ThemeSet::load_defaults();
        let theme_name = &configuration.theme;
        let theme = theme_set
            .themes
            .get(theme_name.as_str())
            .expect("theme")
            .to_owned();

        Self {
            color_mode,
            syntax_set,
            syntax,
            theme,
        }
    }
}

impl Helper for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        let hinter = HistoryHinter {};
        hinter.hint(line, pos, ctx)
    }
}

impl Validator for ReplHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let _ = ctx;
        Ok(ValidationResult::Valid(None))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let configuration = Configuration::default();
        let helper = ReplHelper::new(&configuration);

        assert_eq!(helper.color_mode, ColorMode::Forced);
        assert!(helper.syntax_set.find_syntax_by_name("SQL").is_some());
        assert_eq!(helper.syntax.name, "SQL");
        assert_eq!(helper.theme.name, Some("Solarized (dark)".to_string()));
    }
}
