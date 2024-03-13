use crate::shell::command::{CommandOptions, LoopCondition, Result, ShellCommand};
use anyhow::bail;
use async_trait::async_trait;
use num_format::Locale;
use std::str::FromStr;

pub(crate) struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self) -> &'static str {
        "locale"
    }

    fn args(&self) -> &'static str {
        "[locale]"
    }

    fn description(&self) -> &'static str {
        "Set the display locale"
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        if options.input.len() <= 1 {
            writeln!(
                options.output,
                "Locale: {}",
                options.configuration.locale.name()
            )?;
            return Ok(LoopCondition::Continue);
        }

        let locale = options.input[1];
        let locale = match Locale::from_str(locale) {
            Ok(locale) => locale,
            Err(_) => bail!("Invalid locale: {locale}"),
        };

        options.configuration.locale = locale;

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::driver::MockConnection;
    use crate::shell::command::LoopCondition;
    use crate::shell::command::{CommandManager, CommandOptions};
    use rustyline::history::DefaultHistory;
    use std::default;

    #[tokio::test]
    async fn test_execute_no_args() -> Result<()> {
        let mut output = Vec::new();
        let configuration = &mut Configuration {
            locale: Locale::en,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".locale"],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let locale_output = String::from_utf8(output)?;
        assert_eq!(locale_output, "Locale: en\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_set_on() -> Result<()> {
        let configuration = &mut Configuration {
            locale: Locale::en,
            ..default::Default::default()
        };
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration,
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".locale", "en-GB"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(configuration.locale, Locale::en_GB);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_invalid_option() {
        let options = CommandOptions {
            command_manager: &CommandManager::default(),
            configuration: &mut Configuration::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".locale", "foo"],
            output: &mut Vec::new(),
        };

        let result = Command.execute(options).await;

        assert!(result.is_err());
    }
}
