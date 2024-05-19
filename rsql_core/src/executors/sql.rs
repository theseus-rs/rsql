use crate::commands::LoopCondition;
use crate::configuration::Configuration;
use crate::executors::Result;
use indicatif::ProgressStyle;
use rsql_drivers::{Connection, LimitQueryResult};
use rsql_formatters;
use rsql_formatters::writers::Output;
use rsql_formatters::{FormatterManager, Results};
use std::fmt;
use std::fmt::Debug;
use tracing::{instrument, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

/// A SQL executor for interacting with a database.
pub(crate) struct SqlExecutor<'a> {
    configuration: &'a Configuration,
    formatter_manager: &'a FormatterManager,
    connection: &'a mut dyn Connection,
    output: &'a mut Output,
}

/// Implementation for [SqlExecutor].
impl<'a> SqlExecutor<'a> {
    pub(crate) fn new(
        configuration: &'a Configuration,
        formatter_manager: &'a FormatterManager,
        connection: &'a mut dyn Connection,
        output: &'a mut Output,
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
                return Err(rsql_formatters::Error::UnknownFormat {
                    format: result_format.to_string(),
                }
                .into())
            }
        };
        let mut options = self.configuration.get_formatter_options();

        let limit = self.configuration.results_limit;
        let mut results = self.execute_sql(sql, limit).await?;
        options.elapsed = start.elapsed();

        formatter
            .format(&options, &mut results, self.output)
            .await?;
        Ok(LoopCondition::Continue)
    }

    /// Execute the SQL and return the results.
    ///
    /// This function is split out so that it can be instrumented and a visual progress indicator
    /// can be shown without leaving artifacts in the output when the results are formatted.
    #[instrument(skip(sql, limit))]
    async fn execute_sql(&mut self, sql: &str, limit: usize) -> Result<Results> {
        Span::current().pb_set_style(&ProgressStyle::with_template(
            "{span_child_prefix}{spinner}",
        )?);
        let command = if sql.len() > 6 { &sql[..6] } else { "" };

        let results = if command.to_lowercase() == "select" {
            let query_results = self.connection.query(sql).await?;

            if limit == 0 {
                Results::Query(query_results)
            } else {
                let limit_query_result = LimitQueryResult::new(query_results, limit);
                Results::Query(Box::new(limit_query_result))
            }
        } else {
            Results::Execute(self.connection.execute(sql).await?)
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
    use mockall::predicate::eq;
    use rsql_drivers::{MemoryQueryResult, MockConnection};

    #[tokio::test]
    async fn test_debug() {
        let configuration = Configuration::default();
        let formatter_manager = FormatterManager::default();
        let mut connection = MockConnection::new();
        let output = &mut Output::default();

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
        let mut output = Output::default();

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
            .returning(|_| Ok(42));
        let connection = &mut connection as &mut dyn Connection;
        let mut output = Output::default();

        let mut executor =
            SqlExecutor::new(&configuration, &formatter_manager, connection, &mut output);
        let result = executor.execute(sql).await?;

        assert_eq!(result, LoopCondition::Continue);
        let execute_output = output.to_string();
        assert!(execute_output.contains("42"));

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_results_query() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let formatter_manager = FormatterManager::default();
        let mut connection = MockConnection::new();
        let sql = "SELECT * FROM foo";
        let limit = 42;
        connection
            .expect_query()
            .returning(|_| Ok(Box::<MemoryQueryResult>::default()));
        let connection = &mut connection as &mut dyn Connection;
        let output = &mut Output::default();

        let mut executor = SqlExecutor::new(&configuration, &formatter_manager, connection, output);

        let results = executor.execute_sql(sql, limit).await?;
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
            .returning(|_| Ok(42));
        let connection = &mut connection as &mut dyn Connection;
        let output = &mut Output::default();

        let mut executor = SqlExecutor::new(&configuration, &formatter_manager, connection, output);

        let results = executor.execute_sql(sql, 0).await?;
        assert!(results.is_execute());
        if let Results::Execute(results) = results {
            assert_eq!(results, 42);
        }

        Ok(())
    }
}
