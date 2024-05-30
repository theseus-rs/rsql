use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_formatters::writers::{ClipboardWriter, FileWriter, StdoutWriter};
use rust_i18n::t;
use std::fs::File;

/// Command to output results to a file or console
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("output_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        let clipboard = t!("output_clipboard", locale = locale);
        let file = t!("output_file", locale = locale);
        t!(
            "output_argument",
            locale = locale,
            clipboard = clipboard,
            file = file
        )
        .to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("output_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let clipboard = t!("output_clipboard", locale = locale).to_string();
        let option = options.input.get(1).unwrap_or(&"".to_string()).to_string();

        if option.is_empty() {
            options.output.set(Box::new(StdoutWriter));
        } else if option == clipboard {
            options.output.set(Box::<ClipboardWriter>::default());
        } else {
            let file = File::create(option)?;
            let writer = FileWriter::new(file);
            options.output.set(Box::new(writer));
        }

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LoopCondition;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::configuration::Configuration;
    use crate::writers::Output;
    use rsql_drivers::{DriverManager, MockConnection};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;
    use tempfile::NamedTempFile;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "output");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "clipboard|<file>");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(
            description,
            "Output the contents to a [file] or the console"
        );
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let mut output = Output::default();
        assert!(output.to_string().is_empty());
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".output".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;
        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(output.to_string(), "stdout");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_clipboard() -> anyhow::Result<()> {
        let mut output = Output::default();
        assert!(output.to_string().is_empty());
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".output".to_string(), "clipboard".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;
        assert_eq!(result, LoopCondition::Continue);
        assert!(output.to_string().contains("Clipboard"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_file() -> anyhow::Result<()> {
        let mut output = Output::default();
        assert!(output.to_string().is_empty());
        let file = NamedTempFile::new()?;
        let path = file.as_ref().to_string_lossy().to_string();
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".output".to_string(), path.clone()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;
        assert_eq!(result, LoopCondition::Continue);
        assert!(output.to_string().contains("File"));
        Ok(())
    }
}
