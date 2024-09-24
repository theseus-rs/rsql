use crate::configuration::Configuration;
use crate::shell::completer::ReplCompleter;
use rsql_drivers::Metadata;
use rsql_formatters::Highlighter;
use rustyline::completion::Completer;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};

pub(crate) struct ReplHelper {
    pub(crate) highlighter: Highlighter,
    pub(crate) completer: ReplCompleter,
}

impl ReplHelper {
    #[cfg(test)]
    pub(crate) fn new(configuration: &Configuration) -> Self {
        Self::new_with_metadata(configuration, Metadata::default())
    }

    pub(crate) fn new_with_metadata(configuration: &Configuration, metadata: Metadata) -> Self {
        let options = configuration.get_formatter_options();
        let highlighter = Highlighter::new(&options, "sql");
        let completer = ReplCompleter::with_config(configuration, metadata);

        Self {
            highlighter,
            completer,
        }
    }
}

impl Helper for ReplHelper {}

impl Completer for ReplHelper {
    type Candidate = <ReplCompleter as Completer>::Candidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, ctx)
    }
}

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
