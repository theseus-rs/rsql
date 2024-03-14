use crate::configuration::Configuration;
use crate::drivers::Results;
use crate::formatters::{ascii, unicode};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::io;
use std::time::Duration;

/// Options for formatters
pub struct FormatterOptions<'a> {
    pub configuration: &'a mut Configuration,
    pub results: &'a Results,
    pub elapsed: &'a Duration,
    pub output: &'a mut (dyn io::Write + Send + Sync),
}

#[async_trait]
pub trait Formatter: Send {
    fn identifier(&self) -> &'static str;
    async fn format<'a>(&self, options: &mut FormatterOptions<'a>) -> Result<()>;
}

/// Manages available formatters
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

        formatter_manager.add(Box::new(ascii::Formatter));
        formatter_manager.add(Box::new(unicode::Formatter));

        formatter_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_manager() {
        let formatter = unicode::Formatter;

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
        assert_eq!(formatters.formats.len(), 2);
    }
}
