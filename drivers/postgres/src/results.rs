use async_trait::async_trait;
use bit_vec::BitVec;
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::TimeZone;
use rsql_driver::Error::{IoError, UnsupportedColumnType};
use rsql_driver::{QueryResult, Result, Value};
use std::time::SystemTime;
use tokio_postgres::types::{FromSql, Type};
use tokio_postgres::{Column, Row};
use uuid::Uuid;

/// Query result that converts tokio-postgres rows to values on demand
pub(crate) struct PostgresQueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl std::fmt::Debug for PostgresQueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresQueryResult")
            .field("columns", &self.columns)
            .field("row_index", &self.row_index)
            .field("row_count", &self.rows.len())
            .finish()
    }
}

impl PostgresQueryResult {
    pub(crate) fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self {
            columns,
            rows,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }

    fn convert_row(row: &Row, buffer: &mut Vec<Value>) -> Result<()> {
        buffer.clear();
        for (index, column) in row.columns().iter().enumerate() {
            let value = convert_to_value(row, column, index)?;
            buffer.push(value);
        }
        Ok(())
    }
}

#[async_trait]
impl QueryResult for PostgresQueryResult {
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

pub(crate) fn convert_to_value(row: &Row, column: &Column, column_index: usize) -> Result<Value> {
    // https://www.postgresql.org/docs/current/datatype.html
    let column_type = column.type_();
    let value = match *column_type {
        Type::BIT | Type::VARBIT => {
            get_single(row, column_index, |v: BitVec| Value::String(bit_string(&v)))?
        }
        Type::BIT_ARRAY | Type::VARBIT_ARRAY => {
            get_array(row, column_index, |v: BitVec| Value::String(bit_string(&v)))?
        }
        Type::BOOL => get_single(row, column_index, |v: bool| Value::Bool(v))?,
        Type::BOOL_ARRAY => get_array(row, column_index, |v: bool| Value::Bool(v))?,
        Type::INT2 => get_single(row, column_index, |v: i16| Value::I16(v))?,
        Type::INT2_ARRAY => get_array(row, column_index, |v: i16| Value::I16(v))?,
        Type::INT4 => get_single(row, column_index, |v: i32| Value::I32(v))?,
        Type::INT4_ARRAY => get_array(row, column_index, |v: i32| Value::I32(v))?,
        Type::INT8 => get_single(row, column_index, |v: i64| Value::I64(v))?,
        Type::INT8_ARRAY => get_array(row, column_index, |v: i64| Value::I64(v))?,
        Type::FLOAT4 => get_single(row, column_index, |v: f32| Value::F32(v))?,
        Type::FLOAT4_ARRAY => get_array(row, column_index, |v: f32| Value::F32(v))?,
        Type::FLOAT8 => get_single(row, column_index, |v: f64| Value::F64(v))?,
        Type::FLOAT8_ARRAY => get_array(row, column_index, |v: f64| Value::F64(v))?,
        Type::TEXT | Type::VARCHAR | Type::CHAR | Type::BPCHAR | Type::NAME => {
            get_single(row, column_index, |v: String| Value::String(v))?
        }
        Type::TEXT_ARRAY | Type::VARCHAR_ARRAY | Type::CHAR_ARRAY | Type::BPCHAR_ARRAY => {
            get_array(row, column_index, |v: String| Value::String(v))?
        }
        Type::NUMERIC => get_single(row, column_index, |v: rust_decimal::Decimal| {
            Value::Decimal(v)
        })?,
        Type::NUMERIC_ARRAY => get_single(row, column_index, |v: rust_decimal::Decimal| {
            Value::Decimal(v)
        })?,
        Type::JSON | Type::JSONB => {
            get_single(row, column_index, |v: serde_json::Value| Value::from(v))?
        }
        Type::JSON_ARRAY | Type::JSONB_ARRAY => {
            get_array(row, column_index, |v: serde_json::Value| Value::from(v))?
        }
        Type::BYTEA => {
            let byte_value: Option<&[u8]> = row
                .try_get(column_index)
                .map_err(|error| IoError(error.to_string()))?;
            match byte_value {
                Some(value) => Value::Bytes(value.to_vec()),
                None => Value::Null,
            }
        }
        Type::DATE => get_single(row, column_index, |v: Date| Value::Date(v))?,
        Type::TIME | Type::TIMETZ => get_single(row, column_index, |v: Time| Value::Time(v))?,
        Type::TIMESTAMP => get_single(row, column_index, |v: DateTime| Value::DateTime(v))?,
        Type::TIMESTAMPTZ => {
            let system_time: Option<SystemTime> = row
                .try_get(column_index)
                .map_err(|error| IoError(error.to_string()))?;
            match system_time {
                Some(value) => {
                    let timestamp = jiff::Timestamp::try_from(value)
                        .map_err(|error| IoError(error.to_string()))?;
                    let date_time = timestamp.to_zoned(TimeZone::UTC).datetime();
                    Value::DateTime(date_time)
                }
                None => Value::Null,
            }
        }
        Type::OID => get_single(row, column_index, |v: u32| Value::U32(v))?,
        Type::OID_ARRAY => get_array(row, column_index, |v: u32| Value::U32(v))?,
        Type::UUID => get_single(row, column_index, |v: Uuid| Value::Uuid(v))?,
        Type::UUID_ARRAY => get_array(row, column_index, |v: Uuid| Value::Uuid(v))?,
        Type::VOID => Value::Null, // pg_sleep() returns void
        _ => {
            return Err(UnsupportedColumnType {
                column_name: column.name().to_string(),
                column_type: column_type.name().to_string(),
            });
        }
    };

    Ok(value)
}

fn get_single<'r, T: FromSql<'r>>(
    row: &'r Row,
    column_index: usize,
    to_value: impl Fn(T) -> Value,
) -> Result<Value> {
    match row
        .try_get::<_, Option<T>>(column_index)
        .map_err(|error| IoError(error.to_string()))?
        .map(to_value)
    {
        Some(value) => Ok(value),
        None => Ok(Value::Null),
    }
}

fn get_array<'r, T: FromSql<'r>>(
    row: &'r Row,
    column_index: usize,
    to_value: impl Fn(T) -> Value,
) -> Result<Value> {
    let original_value_array = row
        .try_get::<_, Option<Vec<T>>>(column_index)
        .map_err(|error| IoError(error.to_string()))?;
    let result = match original_value_array {
        Some(value_array) => {
            let mut values = vec![];
            for value in value_array {
                values.push(to_value(value));
            }
            Value::Array(values)
        }
        None => Value::Null,
    };
    Ok(result)
}

fn bit_string(value: &BitVec) -> String {
    let bit_string: String = value
        .iter()
        .map(|bit| if bit { '1' } else { '0' })
        .collect();
    bit_string
}
