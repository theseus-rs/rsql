use crate::drivers::Results;
use crate::drivers::Results::{Execute, Query};
use crate::formatters::error::Result;
use crate::formatters::FormatterOptions;
use colored::Colorize;
use num_format::{Locale, ToFormattedString};
use std::io::Write;
use std::str::FromStr;

/// Display the footer of the result set.
/// This includes the number of rows returned and the elapsed time.
/// If the timing option is enabled, the elapsed time will be displayed.
/// The number of rows will be formatted based on the locale.
///
/// Example: "N,NNN,NNN rows (M.MMMs)"
pub async fn write_footer<'a>(options: &mut FormatterOptions<'a>, results: &Results) -> Result<()> {
    let configuration = &options.configuration;

    if !configuration.results_footer {
        return Ok(());
    }

    let rows_affected = match results {
        Execute(rows_affected) => *rows_affected,
        Query(query_result) => query_result.rows().await.len() as u64,
    };
    let locale = &configuration.locale;
    let num_locale = Locale::from_str(locale).unwrap_or(Locale::en);
    let rows = rows_affected.to_formatted_string(&num_locale);
    let rows_label = if rows_affected == 1 {
        t!("row", locale = locale, rows = rows).to_string()
    } else {
        t!("rows", locale = locale, rows = rows).to_string()
    };
    let elapsed_display = if configuration.results_timer {
        let elapsed = format!("{:?}", options.elapsed);
        t!("elapsed_format", locale = locale, elapsed = elapsed).to_string()
    } else {
        "".to_string()
    };
    let output = &mut options.output;

    if configuration.color {
        let footer = t!(
            "footer_format",
            locale = locale,
            rows = rows_label,
            elapsed = elapsed_display.dimmed()
        )
        .to_string();
        writeln!(output, "{}", footer)?
    } else {
        let footer = t!(
            "footer_format",
            locale = locale,
            rows = rows_label,
            elapsed = elapsed_display
        )
        .to_string();
        writeln!(output, "{}", footer)?
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configuration::Configuration;
    use crate::drivers::MemoryQueryResult;
    use crate::drivers::{Results, Value};
    use std::io::Cursor;
    use std::time::Duration;

    fn query_result(rows: u8) -> Results {
        let rows: Vec<Vec<Option<Value>>> = (0..rows)
            .map(|_| vec![None, Some(Value::I64(12345))])
            .collect();
        let query_result =
            MemoryQueryResult::new(vec!["id".to_string(), "value".to_string()], rows);
        Query(Box::new(query_result))
    }

    async fn test_write_footer(
        configuration: &mut Configuration,
        results: &Results,
    ) -> anyhow::Result<String> {
        let output = &mut Cursor::new(Vec::new());
        let mut options = FormatterOptions {
            configuration,
            elapsed: Duration::from_nanos(9),
            output,
        };

        write_footer(&mut options, results).await?;

        let output = String::from_utf8(output.get_ref().to_vec())?.replace("\r\n", "\n");
        Ok(output)
    }

    #[tokio::test]
    async fn test_write_footer_disabled() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            results_footer: false,
            ..Default::default()
        };
        let output = test_write_footer(&mut configuration, &query_result(0)).await?;
        assert!(!output.contains("row"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_execute() -> anyhow::Result<()> {
        let mut configuration = Configuration::default();
        let output = test_write_footer(&mut configuration, &Execute(42)).await?;
        assert!(output.contains("42 rows"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_no_rows() -> anyhow::Result<()> {
        let mut configuration = Configuration::default();
        let output = test_write_footer(&mut configuration, &query_result(0)).await?;
        assert!(output.contains("0 rows"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_one_row() -> anyhow::Result<()> {
        let mut configuration = Configuration::default();
        let output = test_write_footer(&mut configuration, &query_result(1)).await?;
        assert!(output.contains("1 row"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_no_color_and_no_timer() -> anyhow::Result<()> {
        let mut configuration = Configuration {
            color: false,
            results_timer: false,
            ..Default::default()
        };
        let output = test_write_footer(&mut configuration, &query_result(1)).await?;
        assert!(output.contains("1 row"));
        assert!(!output.contains("(9ns)"));
        Ok(())
    }
}
