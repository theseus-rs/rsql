use crate::error::Result;
use crate::writers::Output;
use async_trait::async_trait;
use rsql_drivers::QueryResult;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::time::Duration;

/// Options for formatters
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct FormatterOptions {
    pub changes: bool,
    pub color: bool,
    pub elapsed: Duration,
    pub footer: bool,
    pub header: bool,
    pub locale: String,
    pub rows: bool,
    pub theme: String,
    pub timer: bool,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        FormatterOptions {
            changes: true,
            color: true,
            elapsed: Duration::default(),
            footer: true,
            header: true,
            locale: "en".to_string(),
            rows: true,
            theme: "Solarized (dark)".to_string(),
            timer: true,
        }
    }
}

/// Results from a query or execute
#[derive(Debug)]
pub enum Results {
    Query(Box<dyn QueryResult>),
    Execute(u64),
}

impl Results {
    #[must_use]
    pub fn is_query(&self) -> bool {
        matches!(self, Results::Query(_))
    }

    #[must_use]
    pub fn is_execute(&self) -> bool {
        matches!(self, Results::Execute(_))
    }
}

#[async_trait]
pub trait Formatter: Debug + Send + Sync {
    fn identifier(&self) -> &'static str;
    async fn format(
        &self,
        options: &FormatterOptions,
        results: &mut Results,
        output: &mut Output,
    ) -> Result<()>;
}

/// Manages available formatters
#[derive(Debug)]
pub struct FormatterManager {
    formats: BTreeMap<&'static str, Box<dyn Formatter>>,
}

impl FormatterManager {
    /// Create a new instance of the `FormatterManager`
    #[must_use]
    pub fn new() -> Self {
        FormatterManager {
            formats: BTreeMap::new(),
        }
    }

    /// Add a new format to the list of available formatters
    fn add(&mut self, format: Box<dyn Formatter>) {
        let identifier = format.identifier();
        let _ = &self.formats.insert(identifier, format);
    }

    /// Get a formatters by name
    #[must_use]
    pub fn get(&self, identifier: &str) -> Option<&dyn Formatter> {
        self.formats.get(identifier).map(AsRef::as_ref)
    }

    /// Get an iterator over the available formatters
    pub fn iter(&self) -> impl Iterator<Item = &dyn Formatter> {
        self.formats.values().map(AsRef::as_ref)
    }
}

/// Default implementation for the `FormatterManager`
impl Default for FormatterManager {
    fn default() -> Self {
        let mut formatter_manager = FormatterManager::new();

        #[cfg(feature = "ascii")]
        formatter_manager.add(Box::new(crate::ascii::Formatter));
        #[cfg(feature = "csv")]
        formatter_manager.add(Box::new(crate::csv::Formatter));
        #[cfg(feature = "expanded")]
        formatter_manager.add(Box::new(crate::expanded::Formatter));
        #[cfg(feature = "html")]
        formatter_manager.add(Box::new(crate::html::Formatter));
        #[cfg(feature = "json")]
        formatter_manager.add(Box::new(crate::json::Formatter));
        #[cfg(feature = "jsonl")]
        formatter_manager.add(Box::new(crate::jsonl::Formatter));
        #[cfg(feature = "markdown")]
        formatter_manager.add(Box::new(crate::markdown::Formatter));
        #[cfg(feature = "plain")]
        formatter_manager.add(Box::new(crate::plain::Formatter));
        #[cfg(feature = "psql")]
        formatter_manager.add(Box::new(crate::psql::Formatter));
        #[cfg(feature = "sqlite")]
        formatter_manager.add(Box::new(crate::sqlite::Formatter));
        #[cfg(feature = "tsv")]
        formatter_manager.add(Box::new(crate::tsv::Formatter));
        #[cfg(feature = "unicode")]
        formatter_manager.add(Box::new(crate::unicode::Formatter));
        #[cfg(feature = "xml")]
        formatter_manager.add(Box::new(crate::xml::Formatter));
        #[cfg(feature = "yaml")]
        formatter_manager.add(Box::new(crate::yaml::Formatter));

        formatter_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsql_drivers::MemoryQueryResult;

    #[test]
    fn test_results_is_query() {
        let query_results = Box::<MemoryQueryResult>::default();
        assert!(Results::Query(query_results).is_query());
    }

    #[test]
    fn test_results_is_execute() {
        assert!(Results::Execute(42).is_execute());
    }

    #[test]
    fn test_format_manager() {
        let formatter = crate::unicode::Formatter;

        let mut formatter_manager = FormatterManager::new();
        assert_eq!(formatter_manager.formats.len(), 0);

        formatter_manager.add(Box::new(formatter));

        assert_eq!(formatter_manager.formats.len(), 1);
        let result = formatter_manager.get("unicode");
        assert!(result.is_some());

        let mut format_count = 0;
        formatter_manager.iter().for_each(|_command| {
            format_count += 1;
        });
        assert_eq!(format_count, 1);
    }

    #[test]
    fn test_format_manager_default() {
        let formatters = FormatterManager::default();
        let formatter_count = 0;

        #[cfg(feature = "ascii")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "csv")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "expanded")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "html")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "json")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "jsonl")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "markdown")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "plain")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "psql")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "sqlite")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "tsv")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "unicode")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "xml")]
        let formatter_count = formatter_count + 1;
        #[cfg(feature = "yaml")]
        let formatter_count = formatter_count + 1;

        assert_eq!(formatters.formats.len(), formatter_count);
    }
}
