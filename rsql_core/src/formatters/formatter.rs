use crate::configuration::Configuration;
use crate::drivers::Results;
use crate::formatters::error::Result;
use crate::writers::Output;
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::time::Duration;

/// Options for formatters
#[derive(Debug)]
pub struct FormatterOptions<'a> {
    pub configuration: &'a mut Configuration,
    pub elapsed: Duration,
    pub output: &'a mut Output,
}

#[async_trait]
pub trait Formatter: Debug + Send + Sync {
    fn identifier(&self) -> &'static str;
    async fn format<'a>(&self, options: &mut FormatterOptions<'a>, results: &Results)
        -> Result<()>;
}

/// Manages available formatters
#[derive(Debug)]
pub struct FormatterManager {
    formats: BTreeMap<&'static str, Box<dyn Formatter>>,
}

impl FormatterManager {
    /// Create a new instance of the `FormatterManager`
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
    pub fn get(&self, identifier: &str) -> Option<&dyn Formatter> {
        self.formats.get(identifier).map(|format| format.as_ref())
    }

    /// Get an iterator over the available formatters
    pub fn iter(&self) -> impl Iterator<Item = &dyn Formatter> {
        self.formats.values().map(|format| format.as_ref())
    }
}

/// Default implementation for the `FormatterManager`
impl Default for FormatterManager {
    fn default() -> Self {
        let mut formatter_manager = FormatterManager::new();

        formatter_manager.add(Box::new(crate::formatters::ascii::Formatter));
        formatter_manager.add(Box::new(crate::formatters::csv::Formatter));
        formatter_manager.add(Box::new(crate::formatters::html::Formatter));
        formatter_manager.add(Box::new(crate::formatters::json::Formatter));
        formatter_manager.add(Box::new(crate::formatters::jsonl::Formatter));
        formatter_manager.add(Box::new(crate::formatters::markdown::Formatter));
        formatter_manager.add(Box::new(crate::formatters::plain::Formatter));
        formatter_manager.add(Box::new(crate::formatters::psql::Formatter));
        formatter_manager.add(Box::new(crate::formatters::sqlite::Formatter));
        formatter_manager.add(Box::new(crate::formatters::tsv::Formatter));
        formatter_manager.add(Box::new(crate::formatters::unicode::Formatter));
        formatter_manager.add(Box::new(crate::formatters::xml::Formatter));
        formatter_manager.add(Box::new(crate::formatters::yaml::Formatter));

        formatter_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_manager() {
        let formatter = crate::formatters::unicode::Formatter;

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
        assert_eq!(formatters.formats.len(), 13);
    }
}
