use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use rsql_driver::Error::UnsupportedColumnType;
use rsql_driver::{QueryResult, Result, Value};
use sqlx::mysql::{MySqlColumn, MySqlRow};
use sqlx::types::time::OffsetDateTime;
use sqlx::{Column, Row};

/// Query result that converts MySQL rows to values on demand
pub(crate) struct MySqlQueryResult {
    columns: Vec<String>,
    rows: Vec<MySqlRow>,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl std::fmt::Debug for MySqlQueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MySqlQueryResult")
            .field("columns", &self.columns)
            .field("row_index", &self.row_index)
            .field("row_count", &self.rows.len())
            .finish()
    }
}

impl MySqlQueryResult {
    pub(crate) fn new(columns: Vec<String>, rows: Vec<MySqlRow>) -> Self {
        Self {
            columns,
            rows,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }

    fn convert_row(row: &MySqlRow, buffer: &mut Vec<Value>) -> Result<()> {
        buffer.clear();
        for column in row.columns() {
            let value = convert_to_value(row, column)?;
            buffer.push(value);
        }
        Ok(())
    }
}

#[async_trait]
impl QueryResult for MySqlQueryResult {
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

#[expect(clippy::too_many_lines)]
fn convert_to_value(row: &MySqlRow, column: &MySqlColumn) -> Result<Value> {
    let column_name = column.name();

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
    } else if let Ok(value) = row.try_get::<Option<i64>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::I64(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<u64>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::U64(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<f32>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::F32(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<f64>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::F64(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<rust_decimal::Decimal>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::Decimal(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<bool>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::Bool(v)),
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<NaiveDate>, &str>(column_name) {
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
    } else if let Ok(value) = row.try_get::<Option<NaiveTime>, &str>(column_name) {
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
    } else if let Ok(value) = row.try_get::<Option<NaiveDateTime>, &str>(column_name) {
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
    } else if let Ok(value) = row.try_get::<Option<OffsetDateTime>, &str>(column_name) {
        match value {
            Some(v) => {
                let date = v.date();
                let time = v.time();
                let year = i16::try_from(date.year())?;
                let month: u8 = date.month().into();
                let month = i8::try_from(month)?;
                let day = i8::try_from(date.day())?;
                let hour = i8::try_from(time.hour())?;
                let minute = i8::try_from(time.minute())?;
                let second = i8::try_from(time.second())?;
                let nanosecond = i32::try_from(time.nanosecond())?;
                let date_time =
                    jiff::civil::datetime(year, month, day, hour, minute, second, nanosecond);
                Ok(Value::DateTime(date_time))
            }
            None => Ok(Value::Null),
        }
    } else if let Ok(value) = row.try_get::<Option<serde_json::Value>, &str>(column_name) {
        match value {
            Some(v) => Ok(Value::from(v)),
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
