use crate::commands::{CommandManager, CommandOptions, LoopCondition};
use crate::configuration::Configuration;
use crate::executors::{Error, Result};
use regex::Regex;
use rsql_drivers::{Connection, DriverManager};
use rsql_formatters::writers::Output;
use rsql_formatters::FormatterManager;
use rustyline::history::DefaultHistory;
use std::fmt;
use std::fmt::Debug;

/// A command executor.
pub(crate) struct CommandExecutor<'a> {
    configuration: &'a mut Configuration,
    command_manager: &'a CommandManager,
    driver_manager: &'a DriverManager,
    formatter_manager: &'a FormatterManager,
    history: &'a DefaultHistory,
    connection: &'a mut dyn Connection,
    output: &'a mut Output,
}

/// Implementation for [CommandExecutor].
impl<'a> CommandExecutor<'a> {
    pub(crate) fn new(
        configuration: &'a mut Configuration,
        command_manager: &'a CommandManager,
        driver_manager: &'a DriverManager,
        formatter_manager: &'a FormatterManager,
        history: &'a DefaultHistory,
        connection: &'a mut dyn Connection,
        output: &'a mut Output,
    ) -> CommandExecutor<'a> {
        Self {
            configuration,
            command_manager,
            driver_manager,
            formatter_manager,
            history,
            connection,
            output,
        }
    }

    /// Execute the command and return the loop condition.
    pub(crate) async fn execute(&mut self, command: &str) -> Result<LoopCondition> {
        let input = split_string(command);
        let command_identifier = &self.configuration.command_identifier;
        let command_name = &input[0][command_identifier.len()..input[0].len()];
        let locale = &self.configuration.locale;

        let loop_condition = match &self
            .command_manager
            .get_starts_with(locale.as_str(), command_name)
        {
            Some(command) => {
                let options = CommandOptions {
                    configuration: self.configuration,
                    command_manager: self.command_manager,
                    driver_manager: self.driver_manager,
                    formatter_manager: self.formatter_manager,
                    connection: self.connection,
                    history: self.history,
                    input,
                    output: self.output,
                };
                command.execute(options).await?
            }
            None => {
                return Err(Error::InvalidCommand {
                    command_name: command_name.to_string(),
                });
            }
        };

        Ok(loop_condition)
    }
}

impl Debug for CommandExecutor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandExecutor")
            .field("configuration", &self.configuration)
            .field("command_manager", &self.command_manager)
            .field("driver_manager", &self.driver_manager)
            .field("formatter_manager", &self.formatter_manager)
            .field("connection", &self.connection)
            .finish()
    }
}

fn split_string(input: &str) -> Vec<String> {
    let pattern = Regex::new(r#"'[^']*'|"[^"]*"|\S+"#).expect("Invalid regex");
    let mut result = Vec::new();

    for cap in pattern.captures_iter(input) {
        let mut segment = cap[0].to_string();

        if segment.starts_with('"') || segment.starts_with('\'') {
            segment.pop();
            segment.remove(0);
        }

        result.push(segment.replace("\\ ", " "));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsql_drivers::MockConnection;

    #[tokio::test]
    async fn test_debug() {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let output = &mut Output::default();

        let executor = CommandExecutor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            output,
        );
        let debug = format!("{:?}", executor);
        assert!(debug.contains("CommandExecutor"));
        assert!(debug.contains("configuration"));
        assert!(debug.contains("command_manager"));
        assert!(debug.contains("driver_manager"));
        assert!(debug.contains("formatter_manager"));
        assert!(debug.contains("connection"));
    }

    #[tokio::test]
    async fn test_execute_invalid_command() {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let output = &mut Output::default();

        let mut executor = CommandExecutor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            output,
        );

        let result = executor.execute("foo").await;
        assert!(result.is_err());
    }

    async fn test_execute(command_identifier: &str) -> anyhow::Result<()> {
        let mut configuration = Configuration {
            bail_on_error: false,
            command_identifier: command_identifier.to_string(),
            ..Default::default()
        };
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let output = &mut Output::default();

        let mut executor = CommandExecutor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            output,
        );

        let command = format!("{command_identifier}bail on");
        let result = executor.execute(command.as_str()).await?;
        assert_eq!(result, LoopCondition::Continue);
        assert!(configuration.bail_on_error);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_default_command_identifier() -> anyhow::Result<()> {
        test_execute(".").await
    }

    #[tokio::test]
    async fn test_execute_backslash_command_identifier() -> anyhow::Result<()> {
        test_execute("\\").await
    }

    #[tokio::test]
    async fn test_execute_multiple_character_command_identifier() -> anyhow::Result<()> {
        test_execute("!!").await
    }

    fn assert_split(input: &str, expected: Vec<&str>) {
        assert_eq!(split_string(input), expected);
    }

    #[test]
    fn test_split_strings() {
        assert_split(r#"foo "bar baz""#, vec!["foo", "bar baz"]);
        assert_split(r#"foo "bar'baz""#, vec!["foo", "bar'baz"]);

        assert_split(r#"foo 'bar baz'"#, vec!["foo", "bar baz"]);
        assert_split(r#"foo 'bar"baz'"#, vec!["foo", "bar\"baz"]);

        assert_split(
            r#"foo 'bar baz' "qux quux""#,
            vec!["foo", "bar baz", "qux quux"],
        );

        assert_split(r#".print "hello, world!""#, vec![".print", "hello, world!"]);
        assert_split(
            r#"\print "hello, world!""#,
            vec!["\\print", "hello, world!"],
        );
    }
}
