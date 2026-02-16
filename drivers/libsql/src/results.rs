use async_trait::async_trait;
use rsql_driver::Error::IoError;
use rsql_driver::{QueryResult, Result, Row, Value};

/// Query result for the libsql driver
#[derive(Debug)]
pub(crate) struct LibSqlQueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    row_index: usize,
}

impl LibSqlQueryResult {
    pub(crate) fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self {
            columns,
            rows,
            row_index: 0,
        }
    }
}

#[async_trait]
impl QueryResult for LibSqlQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&Row> {
        if self.row_index >= self.rows.len() {
            return None;
        }
        let row = &self.rows[self.row_index];
        self.row_index += 1;
        Some(row)
    }
}

/// Convert a libsql row value at the given column index to an rsql Value
pub(crate) fn convert_to_value(row: &libsql::Row, column_index: i32) -> Result<Value> {
    let value = match row
        .get_value(column_index)
        .map_err(|error| IoError(error.to_string()))?
    {
        libsql::Value::Null => Value::Null,
        libsql::Value::Integer(value) => Value::I64(value),
        libsql::Value::Real(value) => Value::F64(value),
        libsql::Value::Text(value) => Value::String(value),
        libsql::Value::Blob(value) => Value::Bytes(value.clone()),
    };

    Ok(value)
}
