use crate::commands::{CommandOptions, ToggleShellCommand};
use async_trait::async_trait;

/// Command to enable or disable color output
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ToggleShellCommand for Command {
    fn get_name(&self) -> &'static str {
        "color_command"
    }

    fn get_description(&self) -> &'static str {
        "color_description"
    }

    fn get_setting_str(&self) -> &'static str {
        "color_setting"
    }

    fn get_value(&self, options: &CommandOptions<'_>) -> bool {
        options.configuration.color
    }

    fn set_value(&self, options: &mut CommandOptions<'_>, value: bool) {
        options.configuration.color = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{CommandManager, CommandOptions, LoopCondition, ShellCommand};
    use crate::writers::Output;
    use rsql_core::Configuration;
    use rsql_drivers::MockConnection;
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
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".color".to_string(), "foo".to_string()],
            output: &mut Output::default(),
        };
        assert!(Command.execute(options).await.is_err());
    }
}
