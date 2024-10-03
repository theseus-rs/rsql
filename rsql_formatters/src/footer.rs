use crate::error::Result;
use crate::writers::Output;
use crate::Results::{Execute, Query};
use crate::{FormatterOptions, Results};
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
pub async fn write_footer(
    options: &FormatterOptions,
    results: &Results,
    query_rows: u64,
    output: &mut Output,
) -> Result<()> {
    if !options.footer {
        return Ok(());
    }

    let (display_rows, rows_affected) = match results {
        Execute(rows_affected) => (options.changes, *rows_affected),
        Query(_query_result) => (options.rows, query_rows),
    };
    let locale = &options.locale;
    let num_locale = Locale::from_str(locale).unwrap_or(Locale::en);
    let rows = rows_affected.to_formatted_string(&num_locale);
    let rows_label = if !display_rows {
        String::new()
    } else if rows_affected == 1 {
        t!("row", locale = locale, rows = rows).to_string()
    } else {
        t!("rows", locale = locale, rows = rows).to_string()
    };
    let elapsed_display = if options.timer {
        let elapsed = format!("{:?}", options.elapsed);
        t!("elapsed_format", locale = locale, elapsed = elapsed).to_string()
    } else {
        String::new()
    };

    if options.color {
        let footer = t!(
            "footer_format",
            locale = locale,
            rows = rows_label,
            elapsed = elapsed_display.dimmed()
        )
        .trim()
        .to_string();
        writeln!(output, "{footer}")?;
    } else {
        let footer = t!(
            "footer_format",
            locale = locale,
            rows = rows_label,
            elapsed = elapsed_display
        )
        .trim()
        .to_string();
        writeln!(output, "{footer}")?;
    }

    output.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writers::Output;
    use rsql_drivers::{MemoryQueryResult, Value};
    use std::time::Duration;

    fn query_result(rows: u8) -> Results {
        let rows: Vec<Vec<Value>> = (0..rows)
            .map(|_| vec![Value::Null, Value::I64(12345)])
            .collect();
        let query_result =
            MemoryQueryResult::new(vec!["id".to_string(), "value".to_string()], rows);
        Query(Box::new(query_result))
    }

    async fn test_write_footer(
        options: &mut FormatterOptions,
        results: &Results,
        query_rows: u64,
    ) -> anyhow::Result<String> {
        let output = &mut Output::default();
        options.elapsed = Duration::from_nanos(9);

        write_footer(options, results, query_rows, output).await?;

        let output = output.to_string().replace("\r\n", "\n");
        Ok(output)
    }

    #[tokio::test]
    async fn test_write_footer_disabled() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            footer: false,
            ..Default::default()
        };
        let output = test_write_footer(&mut options, &query_result(0), 0).await?;
        assert!(!output.contains("row"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_execute() -> anyhow::Result<()> {
        let mut options = FormatterOptions::default();
        let output = test_write_footer(&mut options, &Execute(42), 0).await?;
        assert!(output.contains("42 rows"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_execute_no_changes() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            changes: false,
            rows: true,
            ..Default::default()
        };
        let output = test_write_footer(&mut options, &Execute(42), 0).await?;
        assert!(!output.contains("42 rows"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_no_rows() -> anyhow::Result<()> {
        let mut options = FormatterOptions::default();
        let output = test_write_footer(&mut options, &query_result(0), 0).await?;
        assert!(output.contains("0 rows"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_one_row() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            changes: false,
            ..Default::default()
        };
        let output = test_write_footer(&mut options, &query_result(1), 1).await?;
        assert!(output.contains("1 row"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_one_row_no_rows_displayed() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            changes: true,
            rows: false,
            ..Default::default()
        };
        let output = test_write_footer(&mut options, &query_result(1), 1).await?;
        assert!(!output.contains("1 row"));
        assert!(output.contains("(9ns)"));
        Ok(())
    }

    #[tokio::test]
    async fn test_write_footer_no_color_and_no_timer() -> anyhow::Result<()> {
        let mut options = FormatterOptions {
            color: false,
            timer: false,
            ..Default::default()
        };
        let output = test_write_footer(&mut options, &query_result(1), 1).await?;
        assert!(output.contains("1 row"));
        assert!(!output.contains("(9ns)"));
        Ok(())
    }
}
