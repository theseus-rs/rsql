use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Run command in a system shell
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("system_command", locale = locale).to_string()
    }

    fn args(&self, locale: &str) -> String {
        t!("system_argument", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("system_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let default = String::new();
        let command_name = options.input.get(1).unwrap_or(&default);
        let mut command = tokio::process::Command::new(command_name);
        let args = options.input.iter().skip(2);

        command.args(args);

        let output = command.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

        if !stdout.is_empty() {
            write!(options.output, "{stdout}")?;
        }

        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        if !stderr.is_empty() {
            write!(options.output, "{stderr}")?;
        }

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::writers::Output;
    use rsql_core::Configuration;
    use rsql_drivers::MockConnection;
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "system");
    }

    #[test]
    fn test_args() {
        let args = Command.args("en");
        assert_eq!(args, "command [args]");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Run command in a system shell");
    }

    #[tokio::test]
    async fn test_execute_no_args() -> anyhow::Result<()> {
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".system".to_string()],
            output: &mut Output::default(),
        };

        let result = Command.execute(options).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_command_no_args() -> anyhow::Result<()> {
        let mut output = Output::default();
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".system".to_string(), "echo".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;
        assert_eq!(result, LoopCondition::Continue);
        let command_output = output.to_string();
        assert_eq!(command_output, "\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_command_with_args() -> anyhow::Result<()> {
        let mut output = Output::default();
        let options = CommandOptions {
            configuration: &mut Configuration::default(),
            command_manager: &CommandManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".system".to_string(), "echo".to_string(), "foo".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;
        assert_eq!(result, LoopCondition::Continue);
        let command_output = output.to_string();
        assert_eq!(command_output, "foo\n");
        Ok(())
    }
}
