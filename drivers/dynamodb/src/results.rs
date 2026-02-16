use async_trait::async_trait;
use aws_sdk_dynamodb::types::AttributeValue;
use indexmap::IndexMap;
use rsql_driver::Error::{IoError, UnsupportedColumnType};
use rsql_driver::{QueryResult, Result, Value};
use std::collections::HashMap;

/// Query result that converts DynamoDB items to values on demand
#[derive(Debug)]
pub(crate) struct DynamoDbQueryResult {
    columns: Vec<String>,
    items: Vec<HashMap<String, AttributeValue>>,
    row_index: usize,
    row_buffer: rsql_driver::Row,
}

impl DynamoDbQueryResult {
    pub(crate) fn new(columns: Vec<String>, items: Vec<HashMap<String, AttributeValue>>) -> Self {
        Self {
            columns,
            items,
            row_index: 0,
            row_buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl QueryResult for DynamoDbQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&rsql_driver::Row> {
        if self.row_index >= self.items.len() {
            return None;
        }
        let item = &self.items[self.row_index];
        self.row_index += 1;
        self.row_buffer.clear();
        for column_name in &self.columns {
            let value = match item.get(column_name) {
                Some(attribute) => convert_to_value(column_name, attribute).ok()?,
                None => Value::Null,
            };
            self.row_buffer.push(value);
        }
        Some(&self.row_buffer)
    }
}

fn convert_to_value(column_name: &str, attribute: &AttributeValue) -> Result<Value> {
    let value = match attribute {
        AttributeValue::B(value) => {
            let value = value.as_ref().to_vec();
            Value::Bytes(value)
        }
        AttributeValue::Bool(value) => Value::Bool(*value),
        AttributeValue::Bs(values) => {
            let values = values
                .iter()
                .map(|value| Value::Bytes(value.as_ref().to_vec()))
                .collect::<Vec<Value>>();
            Value::Array(values)
        }
        AttributeValue::L(values) => {
            let mut items = Vec::new();
            for item in values {
                let value = convert_to_value(column_name, item)?;
                items.push(value);
            }
            Value::Array(items)
        }
        AttributeValue::M(values) => {
            let mut items = IndexMap::new();
            for (key, value) in values {
                let key = Value::String(key.to_string());
                let value = convert_to_value(column_name, value)?;
                items.insert(key, value);
            }
            Value::Map(items)
        }
        AttributeValue::N(value) => {
            if value.contains('.') {
                let value: f64 = value
                    .parse()
                    .map_err(|error| IoError(format!("{error:?}")))?;
                Value::F64(value)
            } else {
                let value: i128 = value
                    .parse()
                    .map_err(|error| IoError(format!("{error:?}")))?;
                Value::I128(value)
            }
        }
        AttributeValue::Ns(values) => {
            let mut items = Vec::new();
            for value in values {
                let value = if value.contains('.') {
                    let value: f64 = value
                        .parse()
                        .map_err(|error| IoError(format!("{error:?}")))?;
                    Value::F64(value)
                } else {
                    let value: i128 = value
                        .parse()
                        .map_err(|error| IoError(format!("{error:?}")))?;
                    Value::I128(value)
                };
                items.push(value);
            }
            Value::Array(items)
        }
        AttributeValue::Null(_value) => Value::Null,
        AttributeValue::S(value) => Value::String(value.to_string()),
        AttributeValue::Ss(values) => {
            let values = values
                .iter()
                .map(|value| Value::String(value.to_string()))
                .collect::<Vec<Value>>();
            Value::Array(values)
        }
        _ => {
            return Err(UnsupportedColumnType {
                column_name: column_name.to_string(),
                column_type: format!("{attribute:?}"),
            });
        }
    };
    Ok(value)
}
