use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to display the available drivers
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("drivers_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("drivers_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let driver_manager = options.driver_manager;

        let list_delimiter = t!("list_delimiter", locale = locale).to_string();
        let drivers: String = driver_manager
            .iter()
            .map(|driver| driver.identifier())
            .collect::<Vec<_>>()
            .join(list_delimiter.as_str());
        let drivers_options = t!("drivers_options", locale = locale, drivers = drivers).to_string();
        writeln!(options.output, "{}", drivers_options)?;

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LoopCondition;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::configuration::Configuration;
    use crate::drivers::{DriverManager, MockConnection};
    use crate::formatters::FormatterManager;
    use crate::writers::Output;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "drivers");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Display available database drivers");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut output = Output::default();
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
            input: vec![".drivers".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let drivers_output = output.to_string();
        let mut drivers: Vec<&str> = Vec::new();

        #[cfg(feature = "libsql")]
        drivers.push("libsql");

        #[cfg(feature = "mysql")]
        drivers.push("mysql");

        #[cfg(feature = "postgresql")]
        drivers.push("postgres");

        #[cfg(feature = "postgresql")]
        drivers.push("postgresql");

        #[cfg(feature = "rusqlite")]
        drivers.push("rusqlite");

        #[cfg(feature = "sqlite")]
        drivers.push("sqlite");

        let available_drivers = drivers.join(", ");

        assert_eq!(
            drivers_output,
            format!("Drivers: {available_drivers}\n").as_str()
        );
        Ok(())
    }
}
