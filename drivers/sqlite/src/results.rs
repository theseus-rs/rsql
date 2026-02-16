use async_trait::async_trait;
use rsql_driver::Error::{IoError, UnsupportedColumnType};
use rsql_driver::{QueryResult, Result, Value};
use sqlx::sqlite::{SqliteColumn, SqliteRow};
use sqlx::{Column, Row, TypeInfo};

/// Query result that converts SQLite rows to values on demand
pub(crate) struct SqliteQueryResult {
    columns: Vec<String>,
    rows: Vec<SqliteRow>,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl std::fmt::Debug for SqliteQueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteQueryResult")
            .field("columns", &self.columns)
            .field("row_index", &self.row_index)
            .field("row_count", &self.rows.len())
            .finish()
    }
}

impl SqliteQueryResult {
    pub(crate) fn new(columns: Vec<String>, rows: Vec<SqliteRow>) -> Self {
        Self {
            columns,
            rows,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }

    fn convert_row(row: &SqliteRow, buffer: &mut Vec<Value>) -> Result<()> {
        buffer.clear();
        for column in row.columns() {
            let value = convert_to_value(row, column)?;
            buffer.push(value);
        }
        Ok(())
    }
}

#[async_trait]
impl QueryResult for SqliteQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&rsql_driver::Row> {
        if self.row_index >= self.rows.len() {
            return None;
        }
        let row = &self.rows[self.row_index];
        self.row_index += 1;
        Self::convert_row(row, &mut self.row_buffer).ok()?;
        Some(&self.row_buffer)
    }
}

pub(crate) fn convert_to_value(row: &SqliteRow, column: &SqliteColumn) -> Result<Value> {
    let column_name = column.name();
    let column_type = column.type_info();
    let column_type_name = column_type.name();

    match column_type_name {
        "TEXT" => {
            return match row
                .try_get(column_name)
                .map_err(|error| IoError(error.to_string()))?
            {
                Some(v) => Ok(Value::String(v)),
                None => Ok(Value::Null),
            };
        }
        // Not currently supported by sqlx
        // "NUMERIC" => {
        //     return match row.try_get(column_name)? {
        //         Some(v) => Ok(Value::String(v)),
        //         None => Ok(Value::Null),
        //     };
        // }
        "INTEGER" => {
            return match row
                .try_get(column_name)
                .map_err(|error| IoError(error.to_string()))?
            {
                Some(v) => Ok(Value::I64(v)),
                None => Ok(Value::Null),
            };
        }
        "REAL" => {
            return match row
                .try_get(column_name)
                .map_err(|error| IoError(error.to_string()))?
            {
                Some(v) => Ok(Value::F64(v)),
                None => Ok(Value::Null),
            };
        }
        "BLOB" => {
            return match row
                .try_get(column_name)
                .map_err(|error| IoError(error.to_string()))?
            {
                Some(v) => Ok(Value::Bytes(v)),
                None => Ok(Value::Null),
            };
        }
        _ => {}
    }

    if let Ok(value) = row.try_get::<Option<String>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::String(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<Vec<u8>>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::Bytes(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<i8>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::I8(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<i16>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::I16(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<i32>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::I32(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<f32>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::F32(v)),
            None => Ok(Value::Null),
        }
    } else {
        let column_type = column.type_info();
        let type_name = format!("{column_type:?}");

        Err(UnsupportedColumnType {
            column_name: column_name.to_string(),
            column_type: type_name,
        })
    }
}
