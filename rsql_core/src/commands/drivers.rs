use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to display the available drivers
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("drivers_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("drivers_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let driver_manager = options.driver_manager;

        write!(options.output, "Available drivers: ")?;
        for (i, driver) in driver_manager.iter().enumerate() {
            if i > 0 {
                write!(options.output, ", ")?;
            }
            write!(options.output, "{}", driver.identifier())?;
        }
        writeln!(options.output)?;

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
    use rustyline::history::DefaultHistory;

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration::default();
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".drivers"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let drivers_output = String::from_utf8(output)?;
        assert_eq!(drivers_output, "Available drivers: postgresql, sqlite\n");
        Ok(())
    }
}
