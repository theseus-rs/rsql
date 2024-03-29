use crate::commands::LoopCondition;
use crate::configuration::Configuration;
use crate::drivers::{Connection, Results};
use crate::executors::Result;
use crate::formatters;
use crate::formatters::{FormatterManager, FormatterOptions};
use indicatif::ProgressStyle;
use std::fmt::Debug;
use std::{fmt, io};
use tracing::{instrument, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

/// A SQL executor for interacting with a database.
pub(crate) struct SqlExecutor<'a> {
    configuration: &'a Configuration,
    formatter_manager: &'a FormatterManager,
    connection: &'a mut dyn Connection,
    output: &'a mut (dyn io::Write + Send + Sync),
}

/// Implementation for [SqlExecutor].
impl<'a> SqlExecutor<'a> {
    pub(crate) fn new(
        configuration: &'a Configuration,
        formatter_manager: &'a FormatterManager,
        connection: &'a mut dyn Connection,
        output: &'a mut (dyn io::Write + Send + Sync),
    ) -> SqlExecutor<'a> {
        Self {
            configuration,
            formatter_manager,
            connection,
            output,
        }
    }

    /// Execute SQL.
    pub(crate) async fn execute(&mut self, sql: &str) -> Result<LoopCondition> {
        let start = std::time::Instant::now();
        let result_format = &self.configuration.results_format;
        let formatter = match self.formatter_manager.get(result_format) {
            Some(formatter) => formatter,
            None => {
                return Err(formatters::Error::UnknownFormat {
                    format: result_format.to_string(),
                }
                .into())
            }
        };
        let results = &self.execute_sql(sql).await?;

        let mut options = FormatterOptions {
            configuration: &mut self.configuration.clone(),
            elapsed: start.elapsed(),
            output: &mut self.output,
        };
        formatter.format(&mut options, results).await?;
        Ok(LoopCondition::Continue)
    }

    /// Execute the SQL and return the results.
    ///
    /// This function is split out so that it can be instrumented and a visual progress indicator
    /// can be shown without leaving artifacts in the output when the results are formatted.
    #[instrument(skip(sql))]
    async fn execute_sql(&mut self, sql: &str) -> Result<Results> {
        Span::current().pb_set_style(&ProgressStyle::with_template(
            "{span_child_prefix}{spinner}",
        )?);
        let command = if sql.len() > 6 { &sql[..6] } else { "" };

        let results = if command.to_lowercase() == "select" {
            self.connection.query(sql).await?
        } else {
            self.connection.execute(sql).await?
        };

        Ok(results)
    }
}

impl Debug for SqlExecutor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqlExecutor")
            .field("configuration", &self.configuration)
            .field("formatter_manager", &self.formatter_manager)
            .field("connection", &self.connection)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::{MemoryQueryResult, MockConnection};
    use mockall::predicate::eq;

    #[tokio::test]
    async fn test_debug() {
        let configuration = Configuration::default();
        let formatter_manager = FormatterManager::default();
        let mut connection = MockConnection::new();
        let output = &mut io::stdout();

        let executor =
            SqlExecutor::new(&configuration, &formatter_manager, &mut connection, output);
        let debug = format!("{:?}", executor);
        assert!(debug.contains("SqlExecutor"));
        assert!(debug.contains("configuration"));
        assert!(debug.contains("formatter_manager"));
        assert!(debug.contains("connection"));
    }

    #[tokio::test]
    async fn test_execute_invalid_formatter() -> anyhow::Result<()> {
        let configuration = Configuration {
            results_format: "invalid".to_string(),
            ..Default::default()
        };
        let formatter_manager = FormatterManager::default();
        let mut connection = MockConnection::new();
        let sql = "SELECT * FROM foo";
        let connection = &mut connection as &mut dyn Connection;
        let mut output: Vec<u8> = Vec::new();

        let mut executor =
            SqlExecutor::new(&configuration, &formatter_manager, connection, &mut output);
        let result = executor.execute(sql).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let formatter_manager = FormatterManager::default();
        let mut connection = MockConnection::new();
        let sql = "INSERT INTO foo";
        connection
            .expect_execute()
            .with(eq(sql))
            .returning(|_| Ok(Results::Execute(42)));
        let connection = &mut connection as &mut dyn Connection;
        let mut output: Vec<u8> = Vec::new();

        let mut executor =
            SqlExecutor::new(&configuration, &formatter_manager, connection, &mut output);
        let result = executor.execute(sql).await?;

        assert_eq!(result, LoopCondition::Continue);
        let execute_output = String::from_utf8(output)?;
        assert!(execute_output.contains("42"));

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_results_query() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let formatter_manager = FormatterManager::default();
        let mut connection = MockConnection::new();
        let sql = "SELECT * FROM foo";
        connection
            .expect_query()
            .with(eq(sql))
            .returning(|_| Ok(Results::Query(Box::new(MemoryQueryResult::default()))));
        let connection = &mut connection as &mut dyn Connection;
        let output = &mut io::stdout();

        let mut executor = SqlExecutor::new(&configuration, &formatter_manager, connection, output);

        let results = executor.execute_sql(sql).await?;
        assert!(results.is_query());

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_results_execute() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let formatter_manager = FormatterManager::default();
        let mut connection = MockConnection::new();
        let sql = "INSERT INTO foo";
        connection
            .expect_execute()
            .with(eq(sql))
            .returning(|_| Ok(Results::Execute(42)));
        let connection = &mut connection as &mut dyn Connection;
        let output = &mut io::stdout();

        let mut executor = SqlExecutor::new(&configuration, &formatter_manager, connection, output);

        let results = executor.execute_sql(sql).await?;
        assert!(results.is_execute());
        if let Results::Execute(results) = results {
            assert_eq!(results, 42);
        }

        Ok(())
    }
}
