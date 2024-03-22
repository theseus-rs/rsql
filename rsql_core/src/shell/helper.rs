use crate::configuration::Configuration;
use crate::formatters::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};

pub(crate) struct ReplHelper {
    pub(crate) highlighter: Highlighter,
}

impl ReplHelper {
    pub(crate) fn new(configuration: &Configuration) -> Self {
        let highlighter = Highlighter::new(configuration, "sql");

        Self { highlighter }
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
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_new() {
        let configuration = Configuration::default();
        let _ = ReplHelper::new(&configuration);
    }

    #[test]
    fn test_hinter() {
        let configuration = Configuration::default();
        let helper = ReplHelper::new(&configuration);
        let history = &DefaultHistory::new();
        let ctx = Context::new(history);
        let hint = helper.hint("SELECT", 0, &ctx);
        assert!(hint.is_none());
    }
}
