use async_trait::async_trait;
use jiff::civil::{Date, DateTime};
use rsql_driver::Error::IoError;
use rsql_driver::{QueryResult, Result, Value};
use serde_json::Value as JsonValue;
use uuid::Uuid;

/// Query result that converts ClickHouse JSON rows to values on demand
#[derive(Debug)]
pub(crate) struct ClickHouseQueryResult {
    columns: Vec<String>,
    column_types: Vec<(Option<String>, String)>,
    data_rows: Vec<JsonValue>,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl ClickHouseQueryResult {
    pub(crate) fn new(
        columns: Vec<String>,
        column_types: Vec<(Option<String>, String)>,
        data_rows: Vec<JsonValue>,
    ) -> Self {
        Self {
            columns,
            column_types,
            data_rows,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl QueryResult for ClickHouseQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&rsql_driver::Row> {
        if self.row_index >= self.data_rows.len() {
            return None;
        }
        let row_json = &self.data_rows[self.row_index];
        self.row_index += 1;
        let row_data = row_json.as_object()?;
        self.row_buffer.clear();
        for (column_name, column_type) in self.columns.iter().zip(self.column_types.iter()) {
            let Some(value) = row_data.get(column_name) else {
                self.row_buffer.push(Value::Null);
                continue;
            };
            let borrowed_type = (column_type.0.as_deref(), column_type.1.as_str());
            let converted_value = convert_json_to_value(&borrowed_type, value).ok()?;
            self.row_buffer.push(converted_value);
        }
        Some(&self.row_buffer)
    }
}

pub(crate) fn parse_column_type(column_type: &str) -> (Option<&str>, &str) {
    if let Some(start) = column_type.find('(')
        && let Some(end) = column_type.rfind(')')
    {
        let container = &column_type[..start];
        let inner = &column_type[start + 1..end];
        return (Some(container), inner);
    }
    (None, column_type)
}

fn convert_json_to_value(
    column_type: &(Option<&str>, &str),
    json_value: &JsonValue,
) -> Result<Value> {
    let value = match column_type {
        (Some("Array"), column_type) => {
            if let Some(array) = json_value.as_array() {
                let values = array
                    .iter()
                    .map(|item| {
                        let column_type = parse_column_type(column_type);
                        convert_json_to_value(&column_type, item)
                    })
                    .collect::<Result<Vec<Value>>>()?;
                Value::Array(values)
            } else {
                Value::Null
            }
        }
        (_, "Nothing") => Value::Null,
        (_, "Bool") => {
            if let Some(b) = json_value.as_bool() {
                Value::Bool(b)
            } else {
                Value::Null
            }
        }
        (_, "Int8") => {
            if let Some(i) = json_value.as_i64() {
                Value::I8(i as i8)
            } else {
                Value::Null
            }
        }
        (_, "Int16") => {
            if let Some(i) = json_value.as_i64() {
                Value::I16(i as i16)
            } else {
                Value::Null
            }
        }
        (_, "Int32") => {
            if let Some(i) = json_value.as_i64() {
                Value::I32(i as i32)
            } else {
                Value::Null
            }
        }
        (_, "Int64") => {
            if let Some(s) = json_value.as_str() {
                let value = s
                    .parse::<i64>()
                    .map_err(|error| IoError(error.to_string()))?;
                Value::I64(value)
            } else {
                Value::Null
            }
        }
        (_, "Int128") => {
            if let Some(s) = json_value.as_str() {
                let value = s
                    .parse::<i128>()
                    .map_err(|error| IoError(error.to_string()))?;
                Value::I128(value)
            } else {
                Value::Null
            }
        }
        (_, "UInt8") => {
            if let Some(i) = json_value.as_u64() {
                Value::U8(i as u8)
            } else {
                Value::Null
            }
        }
        (_, "UInt16") => {
            if let Some(i) = json_value.as_u64() {
                Value::U16(i as u16)
            } else {
                Value::Null
            }
        }
        (_, "UInt32") => {
            if let Some(i) = json_value.as_u64() {
                Value::U32(i as u32)
            } else {
                Value::Null
            }
        }
        (_, "UInt64") => {
            if let Some(s) = json_value.as_str() {
                let value = s
                    .parse::<u64>()
                    .map_err(|error| IoError(error.to_string()))?;
                Value::U64(value)
            } else {
                Value::Null
            }
        }
        (_, "UInt128") => {
            if let Some(s) = json_value.as_str() {
                let value = s
                    .parse::<u128>()
                    .map_err(|error| IoError(error.to_string()))?;
                Value::U128(value)
            } else {
                Value::Null
            }
        }
        (_, "Float32") => {
            if let Some(f) = json_value.as_f64() {
                Value::F32(f as f32)
            } else {
                Value::Null
            }
        }
        (_, "Float64") => {
            if let Some(f) = json_value.as_f64() {
                Value::F64(f)
            } else {
                Value::Null
            }
        }
        (_, "String" | "FixedString") => {
            if let Some(s) = json_value.as_str() {
                Value::String(s.to_string())
            } else {
                Value::Null
            }
        }
        (_, "Date") => {
            if let Some(s) = json_value.as_str() {
                let date =
                    Date::strptime("%Y-%m-%d", s).map_err(|error| IoError(error.to_string()))?;
                Value::Date(date)
            } else {
                Value::Null
            }
        }
        (_, "DateTime") => {
            if let Some(s) = json_value.as_str() {
                let date_time = DateTime::strptime("%Y-%m-%d %H:%M:%S", s)
                    .map_err(|error| IoError(error.to_string()))?;
                Value::DateTime(date_time)
            } else {
                Value::Null
            }
        }
        (_, "UUID") => {
            if let Some(s) = json_value.as_str() {
                let uuid = Uuid::parse_str(s).map_err(|error| IoError(error.to_string()))?;
                Value::Uuid(uuid)
            } else {
                Value::Null
            }
        }
        _ => {
            return Err(IoError(format!("Unsupported data type: {column_type:?}")));
        }
    };
    Ok(value)
}
