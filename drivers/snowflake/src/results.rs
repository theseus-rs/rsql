use crate::SnowflakeError;
use async_trait::async_trait;
use jiff::civil::{Date, DateTime, Time};
use rsql_driver::Error::IoError;
use rsql_driver::{QueryResult, Result, Value};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub(crate) struct ColumnDefinition {
    pub name: String,
    snowflake_type: String,
    scale: Option<u64>,
}

impl ColumnDefinition {
    pub(crate) fn new(name: String, snowflake_type: String, scale: Option<u64>) -> Self {
        Self {
            name,
            snowflake_type,
            scale,
        }
    }

    fn translate_error(value: &str, error: impl Display) -> SnowflakeError {
        SnowflakeError::ResponseContent(format!("could not parse {value}: {error}"))
    }

    pub(crate) fn convert_to_value(&self, value: &serde_json::Value) -> Result<Value> {
        if let serde_json::Value::Null = value {
            return Ok(Value::Null);
        }
        let value = value
            .as_str()
            .ok_or(SnowflakeError::ResponseContent(format!(
                "row data contained non-string value before parsing {value}"
            )))
            .map_err(|error| IoError(error.to_string()))?;
        Ok(match self.snowflake_type.to_lowercase().as_str() {
            "fixed" => {
                if self.scale.is_some() && self.scale.unwrap_or(0) > 0 {
                    Value::F64(
                        value
                            .parse()
                            .map_err(|e| Self::translate_error(value, e))
                            .map_err(|error| IoError(error.to_string()))?,
                    )
                } else {
                    Value::I64(
                        value
                            .parse()
                            .map_err(|e| Self::translate_error(value, e))
                            .map_err(|error| IoError(error.to_string()))?,
                    )
                }
            }
            "boolean" => Value::Bool(
                value
                    .parse()
                    .map_err(|e| Self::translate_error(value, e))
                    .map_err(|error| IoError(error.to_string()))?,
            ),
            "date" => Value::Date(Date::from_str(value)?),
            "time" => Value::Time(Time::from_str(value)?),
            "timestamp_ntz" | "timestamp_ltz" | "timestamp_tz" => {
                Value::DateTime(DateTime::from_str(value)?)
            }
            // includes "text" field for VARCHARs
            _ => Value::String(value.to_string()),
        })
    }

    pub(crate) fn try_from_value(value: &serde_json::Value) -> Result<Self> {
        let name = value["name"]
            .as_str()
            .ok_or(SnowflakeError::ResponseContent(
                "missing column name in response".into(),
            ))
            .map_err(|error| IoError(error.to_string()))?
            .to_string();
        let snowflake_type = value["type"]
            .as_str()
            .ok_or(SnowflakeError::ResponseContent(
                "missing column type in response".into(),
            ))
            .map_err(|error| IoError(error.to_string()))?
            .to_string();
        let scale = value["scale"].as_u64();

        Ok(Self::new(name, snowflake_type, scale))
    }
}

/// Query result that converts Snowflake JSON rows to values on demand
#[derive(Debug)]
pub(crate) struct SnowflakeQueryResult {
    columns: Vec<String>,
    column_definitions: Vec<ColumnDefinition>,
    data_rows: Vec<serde_json::Value>,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl SnowflakeQueryResult {
    pub(crate) fn new(
        columns: Vec<String>,
        column_definitions: Vec<ColumnDefinition>,
        data_rows: Vec<serde_json::Value>,
    ) -> Self {
        Self {
            columns,
            column_definitions,
            data_rows,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl QueryResult for SnowflakeQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&rsql_driver::Row> {
        if self.row_index >= self.data_rows.len() {
            return None;
        }
        let row_json = &self.data_rows[self.row_index];
        self.row_index += 1;
        let row_array = row_json.as_array()?;
        self.row_buffer.clear();
        for (value, column_def) in row_array.iter().zip(self.column_definitions.iter()) {
            let converted = column_def.convert_to_value(value).ok()?;
            self.row_buffer.push(converted);
        }
        Some(&self.row_buffer)
    }
}
