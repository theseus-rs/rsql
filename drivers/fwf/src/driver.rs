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
use tokio::fs::read_to_string;
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

        let fwf_content = read_to_string(&file_name).await?;
        let lines = fwf_content.lines();
        let mut columns = vec![Vec::<String>::new(); widths.len()];

        for line in lines {
            let mut data = widths
                .iter()
                .scan(0, |start, &width| {
                    let end = *start + width as usize;
                    let value = &line[*start..end];
                    *start = end;
                    Some(value.trim().to_string())
                })
                .collect::<Vec<String>>();

            data.reverse();
            for column in &mut columns {
                let column_data = data.pop().expect("data");
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

        let data_frame = DataFrame::new(columns).map_err(|error| IoError(error.to_string()))?;

        let table_name = rsql_driver_polars::get_table_name(file_name)?;
        let mut context = SQLContext::new();
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
mod test {
    use super::*;
    use rsql_driver::{Driver, Value};
    use rsql_driver_test_utils::dataset_url;

    fn database_url() -> String {
        dataset_url("fwf", "users.fwf")
    }

    #[tokio::test]
    async fn test_driver_connect() -> Result<()> {
        let database_url = format!("{}?widths=4,15", database_url());
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url).await?;
        assert_eq!(&database_url, connection.url());
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> Result<()> {
        let database_url = format!("{}?widths=4,15&headers=id,name", database_url());
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url).await?;

        let mut query_result = connection
            .query("SELECT id, name FROM users ORDER BY id")
            .await?;

        assert_eq!(query_result.columns().await, vec!["id", "name"]);
        assert_eq!(
            query_result.next().await,
            Some(vec![
                Value::String("1".to_string()),
                Value::String("John Doe".to_string())
            ])
        );
        assert_eq!(
            query_result.next().await,
            Some(vec![
                Value::String("2".to_string()),
                Value::String("Jane Smith".to_string())
            ])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }
}
