extern crate colored;
extern crate unicode_width;

use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use colored::*;
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

/// Show the help message
#[derive(Debug, Default)]
pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("help_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("help_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let output = options.output;
        let command_identifier = &options.configuration.command_identifier;
        let command_manager = options.command_manager;
        let locale = &options.configuration.locale.as_str();
        let is_cjk =
            locale.starts_with("zh") || locale.starts_with("ja") || locale.starts_with("ko");
        let command_identifier_width = if is_cjk {
            command_identifier.as_str().width()
        } else {
            command_identifier.width_cjk()
        };
        let width = command_manager
            .iter()
            .map(|command| {
                let (command_name_width, command_args_width) = if is_cjk {
                    (command.name(locale).width(), command.args(locale).width())
                } else {
                    (
                        command.name(locale).width_cjk(),
                        command.args(locale).width_cjk(),
                    )
                };

                command_identifier_width + command_name_width + command_args_width
            })
            .max()
            .unwrap_or_default();

        for command in command_manager.iter() {
            let name = command.name(locale);
            let name_width = if is_cjk {
                name.width()
            } else {
                name.width_cjk()
            };
            let args_width = width - name_width;
            let args = if command.args(locale).is_empty() {
                format!(
                    "{args:args_width$}",
                    args = command.args(locale),
                    args_width = args_width
                )
            } else {
                format!(
                    " {args:args_width$}",
                    args = command.args(locale),
                    args_width = args_width - 1
                )
            };
            writeln!(
                output,
                "{name}{args}  {description}",
                name = format!("{command_identifier}{name}").bold(),
                args = args.dimmed(),
                description = command.description(locale),
            )?;
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
    use crate::drivers::{DriverManager, MockConnection};
    use crate::formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    async fn test_execute(command_identifier: &str) -> anyhow::Result<()> {
        let mut configuration = Configuration {
            command_identifier: command_identifier.to_string(),
            ..Default::default()
        };
        let mut output = Vec::new();
        let command = format!("{command_identifier}help");
        let options = CommandOptions {
            configuration: &mut configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![command.as_str()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let help_output = String::from_utf8(output)?;
        assert!(help_output.contains(command.as_str()));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_postgresql_identifier() -> anyhow::Result<()> {
        test_execute("\\").await
    }

    #[tokio::test]
    async fn test_execute_sqlite_identifier() -> anyhow::Result<()> {
        test_execute(".").await
    }
}
