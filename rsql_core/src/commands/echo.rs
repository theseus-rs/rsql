use crate::commands::Error::InvalidOption;
use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use crate::configuration::EchoMode;
use async_trait::async_trait;
use rust_i18n::t;

/// Command to enable or disable echoing commands
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("echo_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        let on = t!("on", locale = locale).to_string();
        let prompt = t!("echo_prompt", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();
        t!(
            "echo_argument",
            locale = locale,
            on = on,
            prompt = prompt,
            off = off
        )
        .to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("echo_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let on = t!("on", locale = locale).to_string();
        let prompt = t!("echo_prompt", locale = locale).to_string();
        let off = t!("off", locale = locale).to_string();

        if options.input.len() <= 1 {
            let echo = match options.configuration.echo {
                EchoMode::On => on,
                EchoMode::Prompt => prompt,
                EchoMode::Off => off,
            };
            let echo_setting = t!("echo_setting", locale = locale, echo = echo).to_string();
            writeln!(options.output, "{echo_setting}")?;
            return Ok(LoopCondition::Continue);
        }

        let argument = options.input[1].to_lowercase();
        let echo = if argument == on {
            EchoMode::On
        } else if argument == prompt {
            EchoMode::Prompt
        } else if argument == off {
            EchoMode::Off
        } else {
            return Err(InvalidOption {
                command_name: self.name(locale).to_string(),
                option: argument.to_string(),
            });
        };

        options.configuration.echo = echo;

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
        assert_eq!(name, "echo");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "on|prompt|off");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Enable or disable echoing commands");
    }

    async fn test_execute_no_args(echo: EchoMode) -> anyhow::Result<()> {
        let mut output = Output::default();
        let configuration = &mut Configuration {
            echo: echo.clone(),
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".echo".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let echo_output = output.to_string();

        match echo {
            EchoMode::On => assert_eq!(echo_output, "Echo: on\n"),
            EchoMode::Prompt => assert_eq!(echo_output, "Echo: prompt\n"),
            EchoMode::Off => assert_eq!(echo_output, "Echo: off\n"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_no_args_on() -> anyhow::Result<()> {
        test_execute_no_args(EchoMode::On).await
    }

    #[tokio::test]
    async fn test_execute_no_args_prompt() -> anyhow::Result<()> {
        test_execute_no_args(EchoMode::Prompt).await
    }

    #[tokio::test]
    async fn test_execute_no_args_off() -> anyhow::Result<()> {
        test_execute_no_args(EchoMode::Off).await
    }

    #[tokio::test]
    async fn test_execute_set_on() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            echo: EchoMode::Off,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".echo".to_string(), "on".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.echo, EchoMode::On);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_prompt() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            echo: EchoMode::Off,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".echo".to_string(), "prompt".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.echo, EchoMode::Prompt);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_off() -> anyhow::Result<()> {
        let configuration = &mut Configuration {
            echo: EchoMode::On,
            ..default::Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".echo".to_string(), "off".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.echo, EchoMode::Off);
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
            input: vec![".echo".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
