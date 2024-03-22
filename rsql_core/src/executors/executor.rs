use crate::commands::{CommandManager, LoopCondition};
use crate::configuration::Configuration;
use crate::drivers::{Connection, DriverManager};
use crate::executors::command::CommandExecutor;
use crate::executors::sql::SqlExecutor;
use crate::executors::Result;
use crate::formatters::FormatterManager;
use rustyline::history::DefaultHistory;
use std::fmt::Debug;
use std::{fmt, io};

pub struct Executor<'a> {
    configuration: &'a mut Configuration,
    command_manager: &'a CommandManager,
    driver_manager: &'a DriverManager,
    formatter_manager: &'a FormatterManager,
    history: &'a DefaultHistory,
    connection: &'a mut dyn Connection,
    output: &'a mut (dyn io::Write + Send + Sync),
}

impl<'a> Executor<'a> {
    pub(crate) fn new(
        configuration: &'a mut Configuration,
        command_manager: &'a CommandManager,
        driver_manager: &'a DriverManager,
        formatter_manager: &'a FormatterManager,
        history: &'a DefaultHistory,
        connection: &'a mut dyn Connection,
        output: &'a mut (dyn io::Write + Send + Sync),
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

    pub async fn execute(&mut self, input: &str) -> Result<LoopCondition> {
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
    use crate::drivers::{MockConnection, Results};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_debug() {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let output = &mut io::stdout();

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
    async fn test_execute_empty_input() -> anyhow::Result<()> {
        let mut configuration = Configuration::default();
        let command_manager = CommandManager::default();
        let driver_manager = DriverManager::default();
        let formatter_manager = FormatterManager::default();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();
        let mut output: Vec<u8> = Vec::new();

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
        let mut output: Vec<u8> = Vec::new();

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
        let result = executor.execute(input.as_str()).await?;
        assert_eq!(result, LoopCondition::Continue);
        let execute_output = String::from_utf8(output)?;
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

    async fn test_execute_sql(echo: bool) -> anyhow::Result<()> {
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
        let mut output: Vec<u8> = Vec::new();

        let mut executor = Executor::new(
            &mut configuration,
            &command_manager,
            &driver_manager,
            &formatter_manager,
            &history,
            &mut connection,
            &mut output,
        );

        let result = executor.execute(input).await?;
        assert_eq!(result, LoopCondition::Continue);
        let execute_output = String::from_utf8(output)?;
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
        test_execute_sql(true).await
    }

    #[tokio::test]
    async fn test_execute_sql_echo_off() -> anyhow::Result<()> {
        test_execute_sql(false).await
    }
}
