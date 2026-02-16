use async_trait::async_trait;
use duckdb::types::{TimeUnit, ValueRef};
use jiff::ToSpan;
use jiff::civil::{Date, DateTime, Time};
use rsql_driver::Error::{IoError, UnsupportedColumnType};
use rsql_driver::{QueryResult, Result, Row, Value};
use std::time::Duration;

/// Query result for the duckdb driver
#[derive(Debug)]
pub(crate) struct DuckDbQueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    row_index: usize,
}

impl DuckDbQueryResult {
    pub(crate) fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self {
            columns,
            rows,
            row_index: 0,
        }
    }
}

#[async_trait]
impl QueryResult for DuckDbQueryResult {
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

/// Convert a duckdb row value at the given column index to an rsql Value
pub(crate) fn convert_to_value(
    row: &duckdb::Row,
    column_name: &String,
    column_index: usize,
) -> Result<Value> {
    let value_ref = row
        .get_ref(column_index)
        .map_err(|error| IoError(error.to_string()))?;
    let value = match value_ref {
        ValueRef::Null => Value::Null,
        ValueRef::Boolean(value) => Value::Bool(value),
        ValueRef::TinyInt(value) => Value::I8(value),
        ValueRef::SmallInt(value) => Value::I16(value),
        ValueRef::Int(value) => Value::I32(value),
        ValueRef::BigInt(value) => Value::I64(value),
        ValueRef::HugeInt(value) => Value::I128(value),
        ValueRef::UTinyInt(value) => Value::U8(value),
        ValueRef::USmallInt(value) => Value::U16(value),
        ValueRef::UInt(value) => Value::U32(value),
        ValueRef::UBigInt(value) => Value::U64(value),
        ValueRef::Float(value) => Value::F32(value),
        ValueRef::Double(value) => Value::F64(value),
        ValueRef::Decimal(value) => Value::String(value.to_string()),
        ValueRef::Text(value) => {
            let value = String::from_utf8(value.to_vec())?;
            Value::String(value)
        }
        ValueRef::Blob(value) => Value::Bytes(value.to_vec()),
        ValueRef::Date32(value) => {
            let days = i64::from(value).days();
            let start_date = Date::new(1970, 1, 1)?.checked_add(days)?;
            Value::Date(start_date)
        }
        ValueRef::Time64(unit, value) => {
            let start_time = Time::new(0, 0, 0, 0)?;
            let duration = match unit {
                TimeUnit::Second => Duration::from_secs(u64::try_from(value)?),
                TimeUnit::Millisecond => Duration::from_millis(u64::try_from(value)?),
                TimeUnit::Microsecond => Duration::from_micros(u64::try_from(value)?),
                TimeUnit::Nanosecond => Duration::from_nanos(u64::try_from(value)?),
            };
            let time = start_time.checked_add(duration)?;
            Value::Time(time)
        }
        ValueRef::Timestamp(unit, value) => {
            let start_date = Date::new(1970, 1, 1)?;
            let start_time = Time::new(0, 0, 0, 0)?;
            let start_date_time = DateTime::from_parts(start_date, start_time);
            let duration = match unit {
                TimeUnit::Second => Duration::from_secs(u64::try_from(value)?),
                TimeUnit::Millisecond => Duration::from_millis(u64::try_from(value)?),
                TimeUnit::Microsecond => Duration::from_micros(u64::try_from(value)?),
                TimeUnit::Nanosecond => Duration::from_nanos(u64::try_from(value)?),
            };
            let date_time = start_date_time.checked_add(duration)?;
            Value::DateTime(date_time)
        }
        _ => {
            let data_type = value_ref.data_type();
            return Err(UnsupportedColumnType {
                column_name: column_name.to_string(),
                column_type: data_type.to_string(),
            });
        }
    };

    Ok(value)
}
