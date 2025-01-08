use crate::error::Result;
use crate::polars::Connection;
use crate::url::UrlExtension;
use crate::Error::{ConversionError, IoError};
use async_trait::async_trait;
use calamine::{open_workbook_auto_from_rs, Data, Range, Reader};
use file_type::FileType;
use indexmap::IndexMap;
use polars::io::SerReader;
use polars::prelude::{IntoLazy, JsonReader};
use polars_sql::SQLContext;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Cursor;
use std::num::NonZeroUsize;
use tokio::fs::read;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "excel"
    }

    async fn connect(
        &self,
        url: String,
        _password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let parsed_url = Url::parse(url.as_str())?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();

        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let has_header = query_parameters
            .get("has_header")
            .map_or(true, |v| v == "true");
        let ignore_errors = query_parameters
            .get("ignore_errors")
            .map_or(false, |v| v == "true");
        let infer_schema_length = match query_parameters.get("infer_schema_length") {
            Some(infer_schema_length) => {
                let length = infer_schema_length
                    .parse::<usize>()
                    .map_err(|error| ConversionError(error.to_string()))?;
                if length == 0 {
                    None
                } else {
                    NonZeroUsize::new(length)
                }
            }
            None => NonZeroUsize::new(100),
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

        let mut context = SQLContext::new();
        let data = read(&file_name).await?;
        let mut sheets =
            open_workbook_auto_from_rs(Cursor::new(data)).map_err(|error| IoError(error.into()))?;
        let sheet_names = sheets.sheet_names();

        for sheet_name in &sheet_names {
            let range = sheets
                .worksheet_range(sheet_name)
                .map_err(|error| IoError(error.into()))?;
            let json = range_to_json(&range, has_header, skip_rows, skip_rows_after_header)?;
            let cursor = Cursor::new(json.as_bytes());
            let data_frame = JsonReader::new(cursor)
                .infer_schema_len(infer_schema_length)
                .set_rechunk(true)
                .with_ignore_errors(ignore_errors)
                .finish()?;

            let mut table_name = crate::polars::driver::get_table_name(file_name.clone())?;
            if sheet_names.len() > 1 {
                let sheet_table_suffix = sheet_name
                    .chars()
                    .map(|character| {
                        if character.is_alphanumeric() {
                            character
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>();
                table_name = format!("{table_name}__{sheet_table_suffix}");
            }
            context.register(table_name.as_str(), data_frame.lazy());
        }

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        file_type
            .media_types()
            .contains(&"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    }
}

#[expect(clippy::cast_possible_truncation)]
fn range_to_json(
    range: &Range<Data>,
    has_header: bool,
    skip_rows: usize,
    skip_rows_after_header: usize,
) -> Result<String> {
    let (_height, width) = range.get_size();
    let mut rows = range.rows().skip(skip_rows);
    let headers = if has_header {
        let headers = rows
            .next()
            .ok_or(ConversionError("No header row found".to_string()))?;
        headers
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
    } else {
        (0..width).map(column_name).collect()
    };

    let mut json_rows: Vec<Value> = Vec::new();
    for row in rows.skip(skip_rows_after_header) {
        let mut json_row = IndexMap::new();
        for (column, cell) in row.iter().enumerate() {
            let column_name = headers
                .get(column)
                .ok_or_else(|| ConversionError(format!("Column {column} not found in headers")))?;
            let column_value = match *cell {
                Data::Bool(ref boolean) => Value::Bool(*boolean),
                Data::Empty => Value::Null,
                Data::Float(ref float) => {
                    if float.fract() == 0.0 {
                        json!(*float as i64)
                    } else {
                        json!(float)
                    }
                }
                Data::Int(ref integer) => json!(integer),
                _ => Value::String(cell.to_string()),
            };
            json_row.insert(column_name, column_value);
        }
        json_rows.push(json!(json_row));
    }
    let json = serde_json::to_string(&json_rows).map_err(|error| IoError(error.into()))?;
    Ok(json)
}

#[expect(clippy::cast_possible_truncation)]
/// Generate column names the same as Spreadsheets, A-Z, AA-AZ, BA-BZ, etc.
fn column_name(mut column: usize) -> String {
    let mut name = String::new();
    column += 1;
    while column > 0 {
        column -= 1;
        let value = (column % 26) as u8;
        name.insert(0, char::from(b'A' + value));
        column /= 26;
    }
    name
}

#[cfg(test)]
mod test {
    use crate::test::dataset_url;
    use crate::{DriverManager, Value};

    fn database_url() -> String {
        dataset_url("excel", "users.xlsx")
    }

    #[tokio::test]
    async fn test_driver_connect() -> anyhow::Result<()> {
        let database_url = database_url();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&database_url).await?;
        assert_eq!(&database_url, connection.url());
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> anyhow::Result<()> {
        let database_url = database_url();
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(&database_url).await?;

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