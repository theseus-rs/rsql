use async_trait::async_trait;
use file_type::FileType;
use polars::datatypes::PlSmallStr;
use polars::frame::DataFrame;
use polars::prelude::{Column, IntoLazy, NamedFrom};
use polars::series::Series;
use polars_sql::SQLContext;
use rsql_driver::Error::{ConversionError, IoError};
use rsql_driver::{Result, UrlExtension};
use rsql_driver_polars::Connection;
use std::collections::HashMap;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "fwf"
    }

    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let query_parameters: HashMap<String, String> =
            parsed_url.query_pairs().into_owned().collect();

        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let widths = query_parameters
            .get("widths")
            .ok_or_else(|| IoError("widths parameter is required".to_string()))?
            .split(',')
            .map(|s| {
                s.parse::<u16>()
                    .map_err(|err| ConversionError(err.to_string()))
            })
            .collect::<Result<Vec<u16>>>()?;
        let headers = if let Some(headers) = query_parameters.get("headers") {
            headers
                .split(',')
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        } else {
            (0..widths.len()).map(column_name).collect::<Vec<String>>()
        };
        if headers.len() != widths.len() {
            return Err(IoError(
                "Number of headers does not match number of columns".to_string(),
            ));
        }

        #[cfg(target_family = "wasm")]
        let fwf_content = std::fs::read_to_string(&file_name)?;
        #[cfg(not(target_family = "wasm"))]
        let fwf_content = tokio::fs::read_to_string(&file_name).await?;
        let lines = fwf_content.lines();
        let mut columns = vec![Vec::<String>::new(); widths.len()];

        for line in lines {
            let mut start = 0;
            let mut data = Vec::with_capacity(widths.len());
            for &width in &widths {
                let end = start + usize::from(width);
                let value = line.get(start..end).ok_or_else(|| {
                    IoError(format!(
                        "Fixed-width row is shorter than the configured widths: {line}"
                    ))
                })?;
                data.push(value.trim().to_string());
                start = end;
            }

            data.reverse();
            for column in &mut columns {
                let column_data = data
                    .pop()
                    .ok_or_else(|| IoError("Fixed-width row is missing column data".to_string()))?;
                column.push(column_data);
            }
        }

        let columns = columns
            .into_iter()
            .zip(headers)
            .map(|(column, header)| {
                let series = Series::new(PlSmallStr::from(header), column);
                Column::from(series)
            })
            .collect::<Vec<Column>>();

        let data_frame =
            DataFrame::new_infer_height(columns).map_err(|error| IoError(error.to_string()))?;

        let table_name = rsql_driver_polars::get_table_name(file_name)?;
        let context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        let extensions = file_type.extensions();
        extensions.contains(&"fwf")
    }
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
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "test assertions intentionally panic when verification fails"
    )]
    async fn test_short_row_returns_error() -> Result<()> {
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(b"abc\n")?;
        let mut url = Url::from_file_path(file.path())
            .map_err(|()| IoError("failed to create file URL".to_string()))?;
        url.set_query(Some("widths=2,2"));

        let result = rsql_driver::Driver::connect(&Driver, url.as_str()).await;
        assert!(matches!(result, Err(IoError(message)) if message.contains("shorter")));
        Ok(())
    }
}
