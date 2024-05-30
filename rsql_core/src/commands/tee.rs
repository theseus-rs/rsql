use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rsql_formatters::writers::{ClipboardWriter, FanoutWriter, FileWriter, StdoutWriter};
use rust_i18n::t;
use std::fs::File;

/// Command to output results to a file and the console
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("tee_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        let clipboard = t!("tee_clipboard", locale = locale);
        let file = t!("tee_file", locale = locale);
        t!(
            "tee_argument",
            locale = locale,
            clipboard = clipboard,
            file = file
        )
        .to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("tee_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let clipboard = t!("tee_clipboard", locale = locale).to_string();
        let option = options.input.get(1).unwrap_or(&"".to_string()).to_string();

        if option.is_empty() {
            options.output.set(Box::new(StdoutWriter));
        } else if option == clipboard {
            let writer = FanoutWriter::new(vec![
                Box::new(StdoutWriter),
                Box::<ClipboardWriter>::default(),
            ]);
            options.output.set(Box::new(writer));
        } else {
            let file = File::create(option)?;
            let writer = FanoutWriter::new(vec![
                Box::new(StdoutWriter),
                Box::new(FileWriter::new(file)),
            ]);
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
        assert_eq!(name, "tee");
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
            "Output contents to the system clipboard or a <file>, and the console"
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
            input: vec![".tee".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;
        assert_eq!(output.to_string(), "stdout");
        assert_eq!(result, LoopCondition::Continue);
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
            input: vec![".tee".to_string(), "clipboard".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;
        let output_debug = format!("{:?}", output);
        assert!(output_debug.contains("StdoutWriter"));
        assert!(output_debug.contains("ClipboardWriter"));
        assert_eq!(result, LoopCondition::Continue);
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
            input: vec![".tee".to_string(), path.clone()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;
        assert!(output.to_string().contains("stdout"));
        assert!(output.to_string().contains("File"));
        assert_eq!(result, LoopCondition::Continue);
        Ok(())
    }
}
