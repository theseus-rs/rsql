use crate::commands::{CommandManager, LoopCondition};
use crate::configuration::Configuration;
use crate::executors::command::CommandExecutor;
use crate::executors::sql::SqlExecutor;
use crate::executors::Result;
use regex::Regex;
use rsql_drivers::{Connection, DriverManager};
use rsql_formatters::writers::Output;
use rsql_formatters::FormatterManager;
use rustyline::history::DefaultHistory;
use std::fmt;
use std::fmt::Debug;

pub struct Executor<'a> {
    configuration: &'a mut Configuration,
    command_manager: &'a CommandManager,
    driver_manager: &'a DriverManager,
    formatter_manager: &'a FormatterManager,
    history: &'a DefaultHistory,
    connection: &'a mut dyn Connection,
    output: &'a mut Output,
}

impl<'a> Executor<'a> {
    pub(crate) fn new(
        configuration: &'a mut Configuration,
        command_manager: &'a CommandManager,
        driver_manager: &'a DriverManager,
        formatter_manager: &'a FormatterManager,
        history: &'a DefaultHistory,
        connection: &'a mut dyn Connection,
        output: &'a mut Output,
    ) -> Executor<'a> {
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

    async fn parse_commands(&self, contents: String) -> Result<Vec<String>> {
        let command_identifier = regex::escape(&self.configuration.command_identifier);
        let pattern = format!(r"(?ms)^\s*({}.*?|.*?;|.*)\s*$", command_identifier);
        let regex = Regex::new(pattern.as_str())?;
        let commands: Vec<String> = regex
            .find_iter(contents.as_str())
            .map(|mat| mat.as_str().trim().to_string())
            .collect();
        Ok(commands)
    }

    pub async fn execute(&mut self, input: &str) -> Result<LoopCondition> {
        let input = input.trim();
        let commands = self.parse_commands(input.to_string()).await?;
        for command in commands {
            if let LoopCondition::Exit(exit_code) = &self.execute_command(command.as_str()).await? {
                return Ok(LoopCondition::Exit(*exit_code));
            }
        }

        Ok(LoopCondition::Continue)
    }

    async fn execute_command(&mut self, input: &str) -> Result<LoopCondition> {
        let input = input.trim();

        if input.is_empty() {
            return Ok(LoopCondition::Continue);
        }

        if self.configuration.echo {
            writeln!(&mut self.output, "{}", input)?;
        }

        let command_identifier = &self.configuration.command_identifier;
        let loop_condition = if input.starts_with(command_identifier) {
            let mut executor = CommandExecutor::new(
                self.configuration,
                self.command_manager,
                self.driver_manager,
                self.formatter_manager,
                self.history,
                self.connection,
                self.output,
            );

            executor.execute(input).await?
        } else {
            let mut executor = SqlExecutor::new(
                self.configuration,
                self.formatter_manager,
                self.connection,
                self.output,
            );

            executor.execute(input).await?
        };

        Ok(loop_condition)
    }
}

impl Debug for Executor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Executor")
            .field("configuration", &self.configuration)
            .field("command_manager", &self.command_manager)
            .field("driver_manager", &self.driver_manager)
            .field("formatter_manager", &self.formatter_manager)
            .field("connection", &self.connection)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use mockall::predicate::eq;
    use rsql_drivers::{MockConnection, Results};

    #[tokio::test]
    async fn test_debug() {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let output = &mut Output::default();

        let executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            output,
        );
        let debug = format!("{:?}", executor);
        assert!(debug.contains("Executor"));
        assert!(debug.contains("configuration"));
        assert!(debug.contains("command_manager"));
        assert!(debug.contains("driver_manager"));
        assert!(debug.contains("formatter_manager"));
        assert!(debug.contains("connection"));
    }

    #[tokio::test]
    async fn test_parse_commands_default_command_identifier() -> anyhow::Result<()> {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let mut output = Output::default();

        let executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            &mut output,
        );
        let contents = indoc! {r#"
            .bail on
            SELECT *
            FROM table;
            .timer on
            INSERT INTO table ...;
            .exit 1
            SELECT 1"#};
        let commands = executor.parse_commands(contents.to_string()).await?;

        assert_eq!(commands.len(), 6);
        assert_eq!(commands[0], ".bail on");
        assert_eq!(commands[1], "SELECT *\nFROM table;");
        assert_eq!(commands[2], ".timer on");
        assert_eq!(commands[3], "INSERT INTO table ...;");
        assert_eq!(commands[4], ".exit 1");
        assert_eq!(commands[5], "SELECT 1");
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_commands_backslash_command_identifier() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            command_identifier: "\\".to_string(),
            ..Default::default()
        };
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let mut output = Output::default();

        let executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            &mut output,
        );

        let contents = indoc! {r#"
            \bail on
            SELECT *
            FROM table;
            \timer on
            INSERT INTO table ...;
            \exit 1
        "#};
        let commands = executor.parse_commands(contents.to_string()).await?;

        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], "\\bail on");
        assert_eq!(commands[1], "SELECT *\nFROM table;");
        assert_eq!(commands[2], "\\timer on");
        assert_eq!(commands[3], "INSERT INTO table ...;");
        assert_eq!(commands[4], "\\exit 1");
        Ok(())
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            bail_on_error: false,
            results_timer: false,
            ..Default::default()
        };
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let mut output = Output::default();

        let mut executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            &mut output,
        );

        let input = indoc! {r#"
            .bail on
            .timer on
        "#};
        let _ = executor.execute(input).await?;
        assert!(configuration.bail_on_error);
        assert!(configuration.results_timer);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_empty_input() -> anyhow::Result<()> {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let mut output = Output::default();

        let mut executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            &mut output,
        );

        let result = executor.execute("   ").await?;
        assert_eq!(result, LoopCondition::Continue);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_loop_exit() -> anyhow::Result<()> {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        connection.expect_stop().returning(|| Ok(()));
        let mut output = Output::default();

        let mut executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            &mut output,
        );

        let result = executor.execute(".exit 42").await?;
        assert_eq!(result, LoopCondition::Exit(42));
        Ok(())
    }

    async fn test_execute_command(command_identifier: &str, echo: bool) -> anyhow::Result<()> {
        let mut configuration = Configuration {
            bail_on_error: false,
            command_identifier: command_identifier.to_string(),
            echo,
            ..Default::default()
        };
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let mut output = Output::default();

        let mut executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            &mut output,
        );

        let input = format!("{command_identifier}bail on");
        let result = executor.execute_command(input.as_str()).await?;
        assert_eq!(result, LoopCondition::Continue);
        let execute_output = output.to_string();
        if echo {
            assert!(execute_output.contains(input.as_str()));
        } else {
            assert!(!execute_output.contains(input.as_str()));
        }
        assert!(configuration.bail_on_error);
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_command_echo_on() -> anyhow::Result<()> {
        test_execute_command("!", true).await
    }

    #[tokio::test]
    async fn test_execute_command_echo_off() -> anyhow::Result<()> {
        test_execute_command("\\", false).await
    }

    async fn test_execute_command_sql(echo: bool) -> anyhow::Result<()> {
        let mut configuration = Configuration {
            echo,
            ..Default::default()
        };
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let input = "INSERT INTO foo";
        connection
            .expect_execute()
            .with(eq(input))
            .returning(|_| Ok(Results::Execute(42)));
        let mut output = Output::default();

        let mut executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            &mut output,
        );

        let result = executor.execute_command(input).await?;
        assert_eq!(result, LoopCondition::Continue);
        let execute_output = output.to_string();
        if echo {
            assert!(execute_output.contains(input));
        } else {
            assert!(!execute_output.contains(input));
        }
        assert!(execute_output.contains("42"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_sql_echo_on() -> anyhow::Result<()> {
        test_execute_command_sql(true).await
    }

    #[tokio::test]
    async fn test_execute_sql_echo_off() -> anyhow::Result<()> {
        test_execute_command_sql(false).await
    }
}
