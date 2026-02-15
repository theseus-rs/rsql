use crate::value::ToValue;
use async_trait::async_trait;
use polars::frame::DataFrame;
use rsql_driver::QueryResult;
use std::fmt::{Debug, Formatter};

/// Query result that converts Polars DataFrame rows to values on demand
pub(crate) struct PolarsQueryResult {
    columns: Vec<String>,
    data_frame: DataFrame,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl Debug for PolarsQueryResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolarsQueryResult")
            .field("columns", &self.columns)
            .field("row_index", &self.row_index)
            .field("row_count", &self.data_frame.height())
            .finish()
    }
}

impl PolarsQueryResult {
    pub(crate) fn new(columns: Vec<String>, data_frame: DataFrame) -> Self {
        Self {
            columns,
            data_frame,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl QueryResult for PolarsQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&rsql_driver::Row> {
        if self.row_index >= self.data_frame.height() {
            return None;
        }
        let row_values = self.data_frame.get(self.row_index)?;
        self.row_index += 1;
        self.row_buffer.clear();
        self.row_buffer
            .extend(row_values.iter().map(|v| v.to_value()));
        Some(&self.row_buffer)
    }
}
