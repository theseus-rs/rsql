use async_trait::async_trait;
use rsql_driver::Error::IoError;
use rsql_driver::{QueryResult, Result, Row, Value};
use rusqlite::types::ValueRef;

/// Query result for the rusqlite driver
#[derive(Debug)]
pub(crate) struct RusqliteQueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    row_index: usize,
}

impl RusqliteQueryResult {
    pub(crate) fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self {
            columns,
            rows,
            row_index: 0,
        }
    }
}

#[async_trait]
impl QueryResult for RusqliteQueryResult {
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

/// Convert a rusqlite row value at the given column index to an rsql Value
pub(crate) fn convert_to_value(row: &rusqlite::Row, column_index: usize) -> Result<Value> {
    let value = match row
        .get_ref(column_index)
        .map_err(|error| IoError(error.to_string()))?
    {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(value) => Value::I64(value),
        ValueRef::Real(value) => Value::F64(value),
        ValueRef::Text(value) => {
            let value = String::from_utf8(value.to_vec())?;
            Value::String(value)
        }
        ValueRef::Blob(value) => Value::Bytes(value.to_vec()),
    };

    Ok(value)
}
