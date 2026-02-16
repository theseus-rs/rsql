use async_trait::async_trait;
use bit_vec::BitVec;
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use jiff::civil::{Date, DateTime, Time};
use rsql_driver::Error::{IoError, UnsupportedColumnType};
use rsql_driver::{QueryResult, Result, Value};
use sqlx::postgres::types::Oid;
use sqlx::postgres::{PgColumn, PgRow};
use sqlx::{Column, ColumnIndex, Decode, Row, Type};
use uuid::Uuid;

/// Query result that converts PostgreSQL rows to values on demand
pub(crate) struct PostgreSqlQueryResult {
    columns: Vec<String>,
    rows: Vec<PgRow>,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl std::fmt::Debug for PostgreSqlQueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgreSqlQueryResult")
            .field("columns", &self.columns)
            .field("row_index", &self.row_index)
            .field("row_count", &self.rows.len())
            .finish()
    }
}

impl PostgreSqlQueryResult {
    pub(crate) fn new(columns: Vec<String>, rows: Vec<PgRow>) -> Self {
        Self {
            columns,
            rows,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }

    fn convert_row(row: &PgRow, buffer: &mut Vec<Value>) -> Result<()> {
        buffer.clear();
        for column in row.columns() {
            let value = convert_to_value(row, column)?;
            buffer.push(value);
        }
        Ok(())
    }
}

#[async_trait]
impl QueryResult for PostgreSqlQueryResult {
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
pub(crate) fn convert_to_value(row: &PgRow, column: &PgColumn) -> Result<Value> {
    let column_type = column.type_info();
    let postgresql_type = &**column_type;
    let column_type = format!("{postgresql_type:?}");
    let column_type_parts: Vec<&str> = column_type.split('(').collect();
    let column_name = column.name();

    let Some(column_type_first_part) = column_type_parts.first() else {
        return Err(UnsupportedColumnType {
            column_name: column.name().to_string(),
            column_type: column_type.to_string(),
        });
    };

    let value = match *column_type_first_part {
        "Bool" => get_value(row, column_name, |v: bool| Value::Bool(v))?,
        "BoolArray" => get_value(row, column_name, |v: Vec<bool>| {
            Value::Array(v.into_iter().map(Value::Bool).collect())
        })?,
        "Bpchar" | "Char" | "Name" | "Text" | "Varchar" => {
            get_value(row, column_name, |v: String| Value::String(v))?
        }
        "BpcharArray" | "CharArray" | "NameArray" | "TextArray" | "VarcharArray" => {
            get_value(row, column_name, |v: Vec<String>| {
                Value::Array(v.into_iter().map(Value::String).collect())
            })?
        }
        "Bytea" => get_value(row, column_name, |v: Vec<u8>| Value::Bytes(v.clone()))?,
        "ByteaArray" => get_value(row, column_name, |v: Vec<Vec<u8>>| {
            Value::Array(v.into_iter().map(|v| Value::Bytes(v.clone())).collect())
        })?,
        "Int2" => get_value(row, column_name, |v: i16| Value::I16(v))?,
        "Int2Array" => get_value(row, column_name, |v: Vec<i16>| {
            Value::Array(v.into_iter().map(Value::I16).collect())
        })?,
        "Int4" => get_value(row, column_name, |v: i32| Value::I32(v))?,
        "Int4Array" => get_value(row, column_name, |v: Vec<i32>| {
            Value::Array(v.into_iter().map(Value::I32).collect())
        })?,
        "Int8" => get_value(row, column_name, |v: i64| Value::I64(v))?,
        "Int8Array" => get_value(row, column_name, |v: Vec<i64>| {
            Value::Array(v.into_iter().map(Value::I64).collect())
        })?,
        "Oid" => get_value(row, column_name, |v: Oid| Value::U32(v.0))?,
        "OidArray" => get_value(row, column_name, |v: Vec<Oid>| {
            Value::Array(v.into_iter().map(|v| Value::U32(v.0)).collect())
        })?,
        "Json" | "Jsonb" => get_value(row, column_name, |v: serde_json::Value| Value::from(v))?,
        "JsonArray" | "JsonbArray" => get_value(row, column_name, |v: Vec<serde_json::Value>| {
            Value::Array(v.into_iter().map(Value::from).collect())
        })?,
        // "Point" => Value::Null,
        // "PointArray" => Value::Null,
        // "Lseg" => Value::Null,
        // "LsegArray" => Value::Null,
        // "Path" => Value::Null,
        // "PathArray" => Value::Null,
        // "Box" => Value::Null,
        // "BoxArray" => Value::Null,
        // "Polygon" => Value::Null,
        // "PolygonArray" => Value::Null,
        // "Line" => Value::Null,
        // "LineArray" => Value::Null,
        // "Cidr" => Value::Null,
        // "CidrArray" => Value::Null,
        "Float4" => get_value(row, column_name, |v: f32| Value::F32(v))?,
        "Float4Array" => get_value(row, column_name, |v: Vec<f32>| {
            Value::Array(v.into_iter().map(Value::F32).collect())
        })?,
        "Float8" => get_value(row, column_name, |v: f64| Value::F64(v))?,
        "Float8Array" => get_value(row, column_name, |v: Vec<f64>| {
            Value::Array(v.into_iter().map(Value::F64).collect())
        })?,
        // "Unknown" => Value::Null,
        // "Circle" => Value::Null,
        // "CircleArray" => Value::Null,
        // "Macaddr" => Value::Null,
        // "MacaddrArray" => Value::Null,
        // "Macaddr8" => Value::Null,
        // "Macaddr8Array" => Value::Null,
        // "Inet" => Value::Null,
        // "InetArray" => Value::Null,
        "Date" => get_value(row, column_name, |v: NaiveDate| naive_date_to_value(v))?,
        "DateArray" => get_value(row, column_name, |v: Vec<NaiveDate>| {
            Value::Array(v.into_iter().map(naive_date_to_value).collect())
        })?,
        "Time" | "Timetz" => get_value(row, column_name, |v: NaiveTime| naive_time_to_value(v))?,
        "TimeArray" | "TimetzArray" => get_value(row, column_name, |v: Vec<NaiveTime>| {
            Value::Array(v.into_iter().map(naive_time_to_value).collect())
        })?,
        "Timestamp" => get_value(row, column_name, |v: NaiveDateTime| {
            naive_date_time_to_value(v)
        })?,
        "TimestampArray" => get_value(row, column_name, |v: Vec<NaiveDateTime>| {
            Value::Array(v.into_iter().map(naive_date_time_to_value).collect())
        })?,
        "Timestamptz" => get_value(row, column_name, |v: chrono::DateTime<Utc>| {
            naive_date_time_to_value(v.naive_utc())
        })?,
        "TimestamptzArray" => get_value(row, column_name, |v: Vec<chrono::DateTime<Utc>>| {
            Value::Array(
                v.into_iter()
                    .map(|v| naive_date_time_to_value(v.naive_utc()))
                    .collect(),
            )
        })?,
        // "Interval" => Value::Null,
        // "IntervalArray" => Value::Null,
        "Bit" | "Varbit" => get_value(row, column_name, |v: BitVec| Value::String(bit_string(&v)))?,
        "BitArray" | "VarbitArray" => get_value(row, column_name, |v: Vec<BitVec>| {
            Value::Array(
                v.into_iter()
                    .map(|v| Value::String(bit_string(&v)))
                    .collect(),
            )
        })?,
        "Numeric" => get_value(row, column_name, |v: rust_decimal::Decimal| {
            Value::Decimal(v)
        })?,
        "NumericArray" => get_value(row, column_name, |v: Vec<rust_decimal::Decimal>| {
            Value::Array(v.into_iter().map(Value::Decimal).collect())
        })?,
        // Some(&"Record"Some(& => Value::Null,
        // Some(&"RecordArray") => Value::Null,
        "Uuid" => get_value(row, column_name, |v: Uuid| Value::Uuid(v))?,
        "UuidArray" => get_value(row, column_name, |v: Vec<Uuid>| {
            Value::Array(v.into_iter().map(Value::Uuid).collect())
        })?,
        // "Int4Range" => Value::Null,
        // "Int4RangeArray" => Value::Null,
        // "NumRange" => Value::Null,
        // "NumRangeArray" => Value::Null,
        // "TsRange" => Value::Null,
        // "TsRangeArray" => Value::Null,
        // "TstzRange" => Value::Null,
        // "TstzRangeArray" => Value::Null,
        // "DateRange" => Value::Null,
        // "DateRangeArray" => Value::Null,
        // "Int8Range" => Value::Null,
        // "Int8RangeArray" => Value::Null,
        // "Jsonpath" => Value::Null,
        // "JsonpathArray" => Value::Null,
        // "Money" => Value::Null,
        // "MoneyArray" => Value::Null,
        "Void" => Value::Null, // pg_sleep() returns void
        // "Custom" => Value::Null,
        // "DeclareWithName" => Value::Null,
        // "DeclareWithOid" => Value::Null,
        _ => {
            return Err(UnsupportedColumnType {
                column_name: column.name().to_string(),
                column_type: column_type.to_string(),
            });
        }
    };

    Ok(value)
}

fn get_value<'r, T, I>(row: &'r PgRow, index: I, to_value: impl Fn(T) -> Value) -> Result<Value>
where
    T: Decode<'r, <PgRow as Row>::Database> + Type<<PgRow as Row>::Database>,
    I: ColumnIndex<PgRow>,
{
    match row
        .try_get::<Option<T>, I>(index)
        .map_err(|error| IoError(error.to_string()))?
        .map(to_value)
    {
        Some(value) => Ok(value),
        None => Ok(Value::Null),
    }
}

fn bit_string(value: &BitVec) -> String {
    let bit_string: String = value
        .iter()
        .map(|bit| if bit { '1' } else { '0' })
        .collect();
    bit_string
}

fn naive_date_to_value(date: NaiveDate) -> Value {
    let Ok(year) = i16::try_from(date.year()) else {
        return Value::Null;
    };
    let Ok(month) = i8::try_from(date.month()) else {
        return Value::Null;
    };
    let Ok(day) = i8::try_from(date.day()) else {
        return Value::Null;
    };
    let Ok(date) = Date::new(year, month, day) else {
        return Value::Null;
    };
    Value::Date(date)
}

fn naive_time_to_value(time: NaiveTime) -> Value {
    let Ok(hour) = i8::try_from(time.hour()) else {
        return Value::Null;
    };
    let Ok(minute) = i8::try_from(time.minute()) else {
        return Value::Null;
    };
    let Ok(second) = i8::try_from(time.second()) else {
        return Value::Null;
    };
    let Ok(nanosecond) = i32::try_from(time.nanosecond()) else {
        return Value::Null;
    };
    let Ok(time) = Time::new(hour, minute, second, nanosecond) else {
        return Value::Null;
    };
    Value::Time(time)
}

fn naive_date_time_to_value(date_time: NaiveDateTime) -> Value {
    let Ok(year) = i16::try_from(date_time.year()) else {
        return Value::Null;
    };
    let Ok(month) = i8::try_from(date_time.month()) else {
        return Value::Null;
    };
    let Ok(day) = i8::try_from(date_time.day()) else {
        return Value::Null;
    };
    let Ok(date) = Date::new(year, month, day) else {
        return Value::Null;
    };
    let Ok(hour) = i8::try_from(date_time.hour()) else {
        return Value::Null;
    };
    let Ok(minute) = i8::try_from(date_time.minute()) else {
        return Value::Null;
    };
    let Ok(second) = i8::try_from(date_time.second()) else {
        return Value::Null;
    };
    let Ok(nanosecond) = i32::try_from(date_time.nanosecond()) else {
        return Value::Null;
    };
    let Ok(time) = Time::new(hour, minute, second, nanosecond) else {
        return Value::Null;
    };
    let date_time = DateTime::from_parts(date, time);
    Value::DateTime(date_time)
}
