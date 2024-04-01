use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;
use std::thread::sleep;
use std::time::Duration;

/// Command to sleep for a given number of seconds
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("sleep_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("sleep_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("sleep_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();

        let duration = match options.input.get(1) {
            Some(seconds) => {
                let millis = match seconds.parse::<f64>() {
                    Ok(millis) => (millis * 1000.0) as u64,
                    Err(error) => {
                        return Err(InvalidOption {
                            command_name: self.name(locale).to_string(),
                            option: format!("{:?}", error),
                        });
                    }
                };
                Duration::from_millis(millis)
            }
            None => Duration::from_millis(1000),
        };

        sleep(duration);

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::configuration::Configuration;
    use crate::formatters::FormatterManager;
    use crate::writers::Output;
    use rsql_drivers::{DriverManager, MockConnection};
    use rustyline::history::DefaultHistory;
    use std::time::Instant;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "sleep");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "[seconds]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Sleep for a specified number of seconds");
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".sleep".to_string()],
            output: &mut Output::default(),
        };

        let start = Instant::now();
        let _ = Command.execute(options).await?;
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() > 500);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_sleep_500_millis() -> anyhow::Result<()> {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".sleep".to_string(), ".5".to_string()],
            output: &mut Output::default(),
        };

        let start = Instant::now();
        let _ = Command.execute(options).await?;
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() > 250);
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
            input: vec![".sleep".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
