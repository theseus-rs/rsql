use async_trait::async_trait;
use file_type::FileType;
use polars::io::SerReader;
use polars::prelude::{CsvParseOptions, CsvReadOptions, IntoLazy};
use polars_sql::SQLContext;
use rsql_driver::Error::{ConversionError, IoError};
use rsql_driver::{Result, UrlExtension};
use rsql_driver_polars::Connection;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "delimited"
    }

    async fn connect(
        &self,
        url: &str,
        _password: Option<String>,
    ) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();

        // Read Options
        #[cfg(target_os = "windows")]
        let file_name = if parsed_url.has_host() {
            // On Windows, the host is the drive letter and the path is the file path.
            let host = parsed_url
                .host_str()
                .unwrap_or_default()
                .replace("%3A", ":");
            format!("{host}{}", parsed_url.path())
        } else {
            parsed_url.to_file()?.to_string_lossy().to_string()
        };
        #[cfg(not(target_os = "windows"))]
        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let file = File::open(&file_name)?;
        let has_header = query_parameters
            .get("has_header")
            .is_none_or(|value| value == "true");
        let ignore_errors = query_parameters
            .get("ignore_errors")
            .is_some_and(|value| value == "true");
        let infer_schema_length = match query_parameters.get("infer_schema_length") {
            Some(infer_schema_length) => {
                let length = infer_schema_length
                    .parse::<usize>()
                    .map_err(|error| ConversionError(error.to_string()))?;
                if length == 0 { None } else { Some(length) }
            }
            None => Some(100),
        };
        let skip_rows = query_parameters
            .get("skip_rows")
            .unwrap_or(&"0".to_string())
            .parse::<usize>()
            .map_err(|error| ConversionError(error.to_string()))?;
        let skip_rows_after_header = query_parameters
            .get("skip_rows_after_header")
            .unwrap_or(&"0".to_string())
            .parse::<usize>()
            .map_err(|error| ConversionError(error.to_string()))?;

        // Parse Options
        let eol = match query_parameters.get("eol") {
            Some(eol) => string_to_ascii_char(eol)?,
            None => b'\n',
        };
        let quote = match query_parameters.get("quote") {
            Some(quote) => Some(string_to_ascii_char(quote)?),
            None => None,
        };
        let separator = match query_parameters.get("separator") {
            Some(separator) => string_to_ascii_char(separator)?,
            None => b',',
        };

        let data_frame = CsvReadOptions::default()
            .with_has_header(has_header)
            .with_ignore_errors(ignore_errors)
            .with_infer_schema_length(infer_schema_length)
            .with_skip_rows(skip_rows)
            .with_skip_rows_after_header(skip_rows_after_header)
            .with_parse_options(
                CsvParseOptions::default()
                    .with_eol_char(eol)
                    .with_quote_char(quote)
                    .with_separator(separator),
            )
            .with_rechunk(true)
            .into_reader_with_file_handle(file)
            .finish()
            .map_err(|error| IoError(error.to_string()))?;

        let table_name = rsql_driver_polars::get_table_name(file_name)?;
        let mut context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

fn string_to_ascii_char(value: &String) -> Result<u8> {
    let chars = value.chars().collect::<Vec<char>>();
    if chars.len() != 1 {
        return Err(ConversionError(format!(
            "Invalid character length; expected 1 character: {value}"
        )));
    }
    let char = chars[0];
    if !char.is_ascii() {
        return Err(ConversionError(format!("Invalid character: {char}")));
    }
    u8::try_from(char).map_err(|error| ConversionError(error.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::{Driver, Value};
    use rsql_driver_test_utils::dataset_url;

    fn database_url() -> String {
        let path = dataset_url("delimited", "users.pipe");
        format!("{path}?separator=|")
    }

    #[tokio::test]
    async fn test_driver_connect() -> Result<()> {
        let database_url = database_url();
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url, None).await?;
        assert_eq!(&database_url, connection.url());
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> Result<()> {
        let database_url = database_url();
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url, None).await?;

        let mut query_result = connection
            .query("SELECT id, name FROM users ORDER BY id")
            .await?;

        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
        );
        assert_eq!(
            query_result.next().await,
            Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }
}
