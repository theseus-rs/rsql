use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, Error, LoopCondition, Result, ShellCommand};
use crate::executors::Executor;
use async_trait::async_trait;
use rust_i18n::t;
use std::fs;

/// Command to read a SQL file and execute it
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("read_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("read_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("read_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let file = *options.input.get(1).unwrap_or(&"");
        let contents = fs::read_to_string(file);

        if let Err(error) = contents {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: error.to_string(),
            });
        }

        let mut executor = Executor::new(
            options.configuration,
            options.command_manager,
            options.driver_manager,
            options.formatter_manager,
            options.history,
            options.connection,
            options.output,
        );

        match executor.execute(contents?.as_str()).await {
            Ok(loop_condition) => Ok(loop_condition),
            Err(error) => {
                return Err(Error::IoError(error.into()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LoopCondition;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::configuration::Configuration;
    use crate::drivers;
    use crate::drivers::{DriverManager, MockConnection};
    use crate::formatters::FormatterManager;
    use rustyline::history::DefaultHistory;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "read");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[file]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Read a SQL file and execute it");
    }

    #[tokio::test]
    async fn test_execute_no_args() {
        let mut output = Vec::new();
        let configuration = &mut Configuration::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".read"],
            output: &mut output,
        };

        assert!(Command.execute(options).await.is_err());
    }

    #[tokio::test]
    async fn test_execute_read_file() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        write!(file, ".locale en-GB")?;
        let path = file.as_ref().to_str().expect("Invalid path");

        let configuration = &mut Configuration {
            locale: "en".to_string(),
            ..Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".read", path],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.locale, "en-GB".to_string());
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_error() -> anyhow::Result<()> {
        let mut file = NamedTempFile::new()?;
        write!(file, ".exit 42")?;
        let path = file.as_ref().to_str().expect("Invalid path");

        let configuration = &mut Configuration::default();
        let connection = &mut MockConnection::new();
        connection
            .expect_stop()
            .returning(|| Err(drivers::Error::IoError(anyhow::anyhow!("Error"))));
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection,
            history: &DefaultHistory::new(),
            input: vec![".read", path],
            output: &mut Vec::new(),
        };

        assert!(Command.execute(options).await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".read", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
