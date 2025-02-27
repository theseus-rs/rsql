use crate::commands::{CommandOptions, LoopCondition, Result, ShellCommand};
use async_trait::async_trait;
use rust_i18n::t;

/// Command to display the available drivers
#[derive(Debug, Default)]
pub struct Command;

#[async_trait]
impl ShellCommand for Command {
    fn name(&self, locale: &str) -> String {
        t!("drivers_command", locale = locale).to_string()
    }

    fn description(&self, locale: &str) -> String {
        t!("drivers_description", locale = locale).to_string()
    }

    async fn execute<'a>(&self, options: CommandOptions<'a>) -> Result<LoopCondition> {
        let locale = options.configuration.locale.as_str();
        let driver_manager = options.driver_manager;

        let list_delimiter = t!("list_delimiter", locale = locale).to_string();
        let drivers: String = driver_manager
            .iter()
            .map(rsql_drivers::Driver::identifier)
            .collect::<Vec<_>>()
            .join(list_delimiter.as_str());
        let drivers_options = t!("drivers_options", locale = locale, drivers = drivers).to_string();
        writeln!(options.output, "{drivers_options}")?;

        Ok(LoopCondition::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::LoopCondition;
    use crate::commands::{CommandManager, CommandOptions};
    use crate::configuration::Configuration;
    use crate::writers::Output;
    use rsql_drivers::{DriverManager, MockConnection};
    use rsql_formatters::FormatterManager;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_name() {
        let name = Command.name("en");
        assert_eq!(name, "drivers");
    }

    #[test]
    fn test_description() {
        let description = Command.description("en");
        assert_eq!(description, "Display available database drivers");
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let mut output = Output::default();
        let configuration = &mut Configuration {
            locale: "en".to_string(),
            ..Default::default()
        };
        let options = CommandOptions {
            configuration,
            command_manager: &CommandManager::default(),
            driver_manager: &DriverManager::default(),
            formatter_manager: &FormatterManager::default(),
            connection: &mut MockConnection::new(),
            history: &DefaultHistory::new(),
            input: vec![".drivers".to_string()],
            output: &mut output,
        };

        let result = Command.execute(options).await?;

        assert_eq!(result, LoopCondition::Continue);
        let drivers_output = output.to_string();
        let drivers: Vec<&str> = vec![
            #[cfg(feature = "driver-arrow")]
            "arrow",
            #[cfg(feature = "driver-avro")]
            "avro",
            #[cfg(feature = "driver-cockroachdb")]
            "cockroachdb",
            #[cfg(feature = "driver-csv")]
            "csv",
            #[cfg(feature = "driver-delimited")]
            "delimited",
            #[cfg(feature = "driver-duckdb")]
            "duckdb",
            #[cfg(feature = "driver-excel")]
            "excel",
            #[cfg(feature = "driver-file")]
            "file",
            #[cfg(feature = "driver-http")]
            "http",
            #[cfg(feature = "driver-https")]
            "https",
            #[cfg(feature = "driver-json")]
            "json",
            #[cfg(feature = "driver-jsonl")]
            "jsonl",
            #[cfg(feature = "driver-libsql")]
            "libsql",
            #[cfg(feature = "driver-mariadb")]
            "mariadb",
            #[cfg(feature = "driver-mysql")]
            "mysql",
            #[cfg(feature = "driver-ods")]
            "ods",
            #[cfg(feature = "driver-parquet")]
            "parquet",
            #[cfg(feature = "driver-postgresql")]
            "postgres",
            #[cfg(feature = "driver-postgresql")]
            "postgresql",
            #[cfg(feature = "driver-redshift")]
            "redshift",
            #[cfg(feature = "driver-rusqlite")]
            "rusqlite",
            #[cfg(feature = "driver-snowflake")]
            "snowflake",
            #[cfg(feature = "driver-sqlite")]
            "sqlite",
            #[cfg(feature = "driver-sqlserver")]
            "sqlserver",
            #[cfg(feature = "driver-tsv")]
            "tsv",
            #[cfg(feature = "driver-xml")]
            "xml",
            #[cfg(feature = "driver-yaml")]
            "yaml",
        ];

        let available_drivers = drivers.join(", ");

        assert_eq!(
            drivers_output,
            format!("Drivers: {available_drivers}\n").as_str()
        );
        Ok(())
    }
}
