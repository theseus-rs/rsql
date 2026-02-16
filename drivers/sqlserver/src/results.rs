use async_trait::async_trait;
use chrono::{Datelike, Timelike};
use rsql_driver::Error::UnsupportedColumnType;
use rsql_driver::{QueryResult, Result, Value};
use tiberius::{Column, Row};

/// Query result that converts SQL Server rows to values on demand
pub(crate) struct SqlServerQueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl std::fmt::Debug for SqlServerQueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqlServerQueryResult")
            .field("columns", &self.columns)
            .field("row_index", &self.row_index)
            .field("row_count", &self.rows.len())
            .finish()
    }
}

impl SqlServerQueryResult {
    pub(crate) fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self {
            columns,
            rows,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl QueryResult for SqlServerQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&rsql_driver::Row> {
        if self.row_index >= self.rows.len() {
            return None;
        }
        let row = &self.rows[self.row_index];
        self.row_index += 1;
        self.row_buffer.clear();
        for (index, column) in row.columns().iter().enumerate() {
            let value = convert_to_value(row, column, index).ok()?;
            self.row_buffer.push(value);
        }
        Some(&self.row_buffer)
    }
}

#[expect(clippy::same_functions_in_if_condition)]
#[expect(clippy::too_many_lines)]
fn convert_to_value(row: &Row, column: &Column, index: usize) -> Result<Value> {
    let column_name = column.name();

    if let Ok(value) = row.try_get(index) {
        let value: Option<&str> = value;
        match value {
            Some(v) => Ok(Value::String(v.to_string())),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<&[u8]> = value;
        match value {
            Some(v) => Ok(Value::Bytes(v.to_vec())),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<u8> = value;
        match value {
            Some(v) => Ok(Value::U8(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i16> = value;
        match value {
            Some(v) => Ok(Value::I16(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i32> = value;
        match value {
            Some(v) => Ok(Value::I32(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<i64> = value;
        match value {
            Some(v) => Ok(Value::I64(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<f32> = value;
        match value {
            Some(v) => Ok(Value::F32(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<f64> = value;
        match value {
            Some(v) => Ok(Value::F64(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<bool> = value;
        match value {
            Some(v) => Ok(Value::Bool(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<rust_decimal::Decimal> = value;
        match value {
            Some(v) => Ok(Value::Decimal(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDate> = value;
        match value {
            Some(v) => {
                let year = i16::try_from(v.year())?;
                let month = i8::try_from(v.month())?;
                let day = i8::try_from(v.day())?;
                let date = jiff::civil::date(year, month, day);
                Ok(Value::Date(date))
            }
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveTime> = value;
        match value {
            Some(v) => {
                let hour = i8::try_from(v.hour())?;
                let minute = i8::try_from(v.minute())?;
                let second = i8::try_from(v.second())?;
                let nanosecond = i32::try_from(v.nanosecond())?;
                let time = jiff::civil::time(hour, minute, second, nanosecond);
                Ok(Value::Time(time))
            }
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get(column_name) {
        let value: Option<chrono::NaiveDateTime> = value;
        match value {
            Some(v) => {
                let year = i16::try_from(v.year())?;
                let month = i8::try_from(v.month())?;
                let day = i8::try_from(v.day())?;
                let hour = i8::try_from(v.hour())?;
                let minute = i8::try_from(v.minute())?;
                let second = i8::try_from(v.second())?;
                let nanosecond = i32::try_from(v.nanosecond())?;
                let date_time =
                    jiff::civil::datetime(year, month, day, hour, minute, second, nanosecond);
                Ok(Value::DateTime(date_time))
            }
            None => Ok(Value::Null),
        }
    } else {
        let column_type = format!("{:?}", column.column_type());
        let type_name = format!("{column_type:?}");
        Err(UnsupportedColumnType {
            column_name: column_name.to_string(),
            column_type: type_name,
        })
    }
}
