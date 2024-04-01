use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to enable or disable color output
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("color_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();
        t!("on_off_argument", locale = locale, on = on, off = off).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("color_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let on = t!("on", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();

        if options.input.len() <= 1 {
            let color = if options.configuration.color { on } else { off };
            let color_setting = t!("color_setting", locale = locale, color = color).to_string();
            writeln!(options.output, "{}", color_setting)?;
            return Ok(LoopCondition::Continue);
        }

        let argument = options.input[1].to_lowercase().to_string();
        let color = if argument == on {
            true
        } else if argument == off {
            false
        } else {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: argument,
            });
        };

        options.configuration.color = color;

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
    use std::default;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "color");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "on|off");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Enable or disable color output");
    }

    async fn test_execute_no_args(color: bool) -> anyhow::Result<()> {
        let mut output = Output::default();
        let configuration = &mut Configuration {
            color,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".color".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let color_output = output.to_string();

        if color {
            assert_eq!(color_output, "Color: on\n");
        } else {
            assert_eq!(color_output, "Color: off\n");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_no_args_on() -> anyhow::Result<()> {
        test_execute_no_args(true).await
    }

    #[tokio::test]
    async fn test_execute_no_args_off() -> anyhow::Result<()> {
        test_execute_no_args(false).await
    }

    #[tokio::test]
    async fn test_execute_set_on() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: false,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".color".to_string(), "on".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.color);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            color: true,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".color".to_string(), "off".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert!(!configuration.color);
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
            input: vec![".color".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
