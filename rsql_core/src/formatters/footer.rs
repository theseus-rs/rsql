use crate::drivers::Results::{Execute, Query};
use crate::formatters::error::Result;
use crate::formatters::FormatterOptions;
use colored::Colorize;
use num_format::ToFormattedString;
use rustyline::ColorMode;
use std::io::Write;

/// Display the footer of the result set.
/// This includes the number of rows returned and the elapsed time.
/// If the timing option is enabled, the elapsed time will be displayed.
/// The number of rows will be formatted based on the locale.
///
/// Example: "N,NNN,NNN rows (M.MMMs)"
pub async fn write_footer<'a>(options: &mut FormatterOptions<'a>) -> Result<()> {
    let configuration = &options.configuration;

    if !configuration.results_footer {
        return Ok(());
    }

    let rows_affected = match options.results {
        Execute(rows_affected) => *rows_affected,
        Query(query_result) => query_result.rows().await.len() as u64,
    };
    let row_label = if rows_affected == 1 { "row" } else { "rows" };
    let elapsed_display = if configuration.results_timer {
        format!("({:?})", options.elapsed)
    } else {
        "".to_string()
    };
    let output = &mut options.output;

    match configuration.color_mode {
        ColorMode::Disabled => writeln!(
            output,
            "{} {} {}",
            rows_affected.to_formatted_string(&configuration.locale),
            row_label,
            elapsed_display
        )?,
        _ => writeln!(
            output,
            "{} {} {}",
            rows_affected.to_formatted_string(&configuration.locale),
            row_label,
            elapsed_display.dimmed()
        )?,
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
            results,
            elapsed: &Duration::from_nanos(9),
            output,
        };

        write_footer(&mut options).await?;

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
            color_mode: ColorMode::Disabled,
            results_timer: false,
            ..Default::default()
        };
        let output = test_write_footer(&mut configuration, &query_result(1)).await?;
        assert!(output.contains("1 row"));
        assert!(!output.contains("(9ns)"));
        Ok(())
    }
}
