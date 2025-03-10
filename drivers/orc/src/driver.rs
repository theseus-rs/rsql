use arrow_array::{
    BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use async_trait::async_trait;
use file_type::FileType;
use orc_rust::ArrowReaderBuilder;
use polars::datatypes::PlSmallStr;
use polars::frame::DataFrame;
use polars::prelude::{Column, IntoLazy, NamedFrom};
use polars::series::Series;
use polars_sql::SQLContext;
use rsql_driver::Error::IoError;
use rsql_driver::{Result, UrlExtension};
use rsql_driver_polars::Connection;
use std::fs::File;
use url::Url;

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl rsql_driver::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "orc"
    }

    #[expect(clippy::too_many_lines)]
    async fn connect(&self, url: &str) -> Result<Box<dyn rsql_driver::Connection>> {
        let parsed_url = Url::parse(url)?;
        let file_name = parsed_url.to_file()?.to_string_lossy().to_string();
        let file = File::open(file_name.clone())?;
        let reader = ArrowReaderBuilder::try_new(file)
            .map_err(|error| IoError(format!("{error:?}")))?
            .build();
        let batches = reader.collect::<Result<Vec<_>, _>>().unwrap();
        let mut columns = Vec::<Column>::new();

        for batch in batches {
            let schema = batch.schema();
            let fields = schema.fields();
            for (field, column) in fields.iter().zip(batch.columns()) {
                let field_name = field.name();
                let any_column = column.as_any();
                if let Some(array) = any_column.downcast_ref::<BooleanArray>() {
                    let column = array.iter().collect::<Vec<Option<bool>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<Int8Array>() {
                    let column = array.iter().collect::<Vec<Option<i8>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<Int16Array>() {
                    let column = array.iter().collect::<Vec<Option<i16>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<Int32Array>() {
                    let column = array.iter().collect::<Vec<Option<i32>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<Int64Array>() {
                    let column = array.iter().collect::<Vec<Option<i64>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<UInt8Array>() {
                    let column = array
                        .iter()
                        .map(|value| value.map(u32::from))
                        .collect::<Vec<Option<u32>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<UInt16Array>() {
                    let column = array
                        .iter()
                        .map(|value| value.map(u32::from))
                        .collect::<Vec<Option<u32>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<UInt32Array>() {
                    let column = array.iter().collect::<Vec<Option<u32>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<UInt64Array>() {
                    let column = array.iter().collect::<Vec<Option<u64>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<Float32Array>() {
                    let column = array.iter().collect::<Vec<Option<f32>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<Float64Array>() {
                    let column = array.iter().collect::<Vec<Option<f64>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else if let Some(array) = any_column.downcast_ref::<StringArray>() {
                    let column = array
                        .iter()
                        .map(|value| value.map(ToString::to_string))
                        .collect::<Vec<Option<String>>>();
                    columns.push(Column::from(Series::new(
                        PlSmallStr::from(field_name),
                        column,
                    )));
                } else {
                    return Err(IoError(format!("Unsupported data type {column:?}")));
                };
            }
        }

        let data_frame = DataFrame::new(columns).map_err(|error| IoError(error.to_string()))?;
        let table_name = rsql_driver_polars::get_table_name(file_name)?;
        let mut context = SQLContext::new();
        context.register(table_name.as_str(), data_frame.lazy());

        let connection = Connection::new(url, context).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, file_type: &FileType) -> bool {
        let extensions = file_type.extensions();
        extensions.contains(&"orc")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rsql_driver::{Driver, Value};
    use rsql_driver_test_utils::dataset_url;

    fn database_url() -> String {
        dataset_url("orc", "users.orc")
    }

    #[tokio::test]
    async fn test_driver_connect() -> Result<()> {
        let database_url = database_url();
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url).await?;
        assert_eq!(&database_url, connection.url());
        connection.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_connection_interface() -> Result<()> {
        let database_url = database_url();
        let driver = crate::Driver;
        let mut connection = driver.connect(&database_url).await?;

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
