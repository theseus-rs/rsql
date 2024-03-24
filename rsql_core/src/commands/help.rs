extern crate colored;
extern crate unicode_width;

use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use colored::*;
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

/// Show the help message
#[derive(Debug, Default)]
pub struct Command;

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
        let locale = options.configuration.locale.as_str();
        let width = command_manager
            .iter()
            .map(|command| {
                let command_name_width = command.name(locale).width();
                let command_args_width = command.args(locale).width();
                command_name_width + command_args_width + 1
            })
            .max()
            .unwrap_or_default();

        for command in command_manager.iter() {
            let name = command.name(locale);
            let name_width = name.width();
            let args_width = width - name_width;
            let mut args = if command.args(locale).is_empty() {
                " ".repeat(args_width)
            } else {
                format!(
                    " {args:args_width$}",
                    args = command.args(locale),
                    args_width = args_width - 1
                )
            };

            let mut name = format!("{command_identifier}{name}");
            let description = command.description(locale);
            if options.configuration.color {
                name = name.bold().to_string();
                args = args.dimmed().to_string();
            }
            writeln!(output, "{name}{args}  {description}")?;
        }
        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{footer, LoopCondition};
    use crate::commands::{CommandManager, CommandOptions};
    use crate::configuration::Configuration;
    use crate::drivers::{DriverManager, MockConnection};
    use crate::formatters::FormatterManager;
    use indoc::indoc;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "help");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Show this help message");
    }

    async fn test_execute(
        color: bool,
        command_identifier: &str,
        locale: &str,
        command: &str,
    ) -> anyhow::Result<String> {
        let mut configuration = Configuration {
            color,
            command_identifier: command_identifier.to_string(),
            locale: locale.to_string(),
            ..Default::default()
        };
        let mut command_manager = CommandManager::new();
        command_manager.add(Box::new(footer::Command));
        command_manager.add(Box::new(Command));
        let mut output = Vec::new();
        let command = format!("{command_identifier}{command}");
        let options = CommandOptions {
            configuration: &mut configuration,
            command_manager: &command_manager,
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
        Ok(help_output)
    }

    #[tokio::test]
    async fn test_execute_postgresql_identifier() -> anyhow::Result<()> {
        let _ = test_execute(true, "\\", "en", "help").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_sqlite_identifier() -> anyhow::Result<()> {
        let _ = test_execute(true, ".", "en", "help").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_unicode_format_en_us() -> anyhow::Result<()> {
        let contents = test_execute(false, ".", "us-US", "help").await?;
        let expected = indoc! {r#"
            .footer on|off  Enable or disable result footer
            .help           Show this help message
        "#};
        assert_eq!(contents, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_unicode_format_de_de() -> anyhow::Result<()> {
        let contents = test_execute(false, ".", "de-DE", "hilfe").await?;
        let expected = indoc! {r#"
            .fußzeile ein|aus  Ergebnisfuß aktivieren oder deaktivieren
            .hilfe             Diese Hilfemeldung anzeigen
        "#};
        assert_eq!(contents, expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_rtl() -> anyhow::Result<()> {
        let _ = test_execute(false, ".", "ar-SA", "مساعدة").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_unicode_format_ja_jp() -> anyhow::Result<()> {
        let _ = test_execute(false, ".", "ja-JP", "ヘルプ").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_unicode_format_ko_kr() -> anyhow::Result<()> {
        let _ = test_execute(false, ".", "ko-KR", "도움말").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_unicode_format_zh_cn() -> anyhow::Result<()> {
        let _ = test_execute(false, ".", "zh-CN", "帮助").await?;
        Ok(())
    }
}
