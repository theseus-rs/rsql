use bigdecimal::BigDecimal;
use bigdecimal::num_bigint::BigInt;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rsql_driver::Error::ConversionError;
use rsql_driver::{Result, Value};
use rust_decimal::Decimal;
use scylla::frame::response::result::{CollectionType, ColumnType, NativeType};
use scylla::value::{
    Counter, CqlDate, CqlDecimal, CqlDuration, CqlTime, CqlTimestamp, CqlValue, CqlVarint,
};
use std::str::FromStr;

pub(crate) fn values_to_cql(
    values: &[Value],
    column_types: &[&ColumnType<'_>],
) -> Result<Vec<Option<CqlValue>>> {
    if values.len() != column_types.len() {
        return Err(ConversionError(format!(
            "expected {} CQL parameters, received {}",
            column_types.len(),
            values.len()
        )));
    }
    values
        .iter()
        .zip(column_types)
        .map(|(value, column_type)| {
            if value.is_null() {
                Ok(None)
            } else {
                value_to_cql(value, column_type).map(Some)
            }
        })
        .collect()
}

#[expect(
    clippy::too_many_lines,
    reason = "the match intentionally enumerates every CQL type"
)]
pub(crate) fn value_to_cql(value: &Value, column_type: &ColumnType<'_>) -> Result<CqlValue> {
    if value.is_null() {
        return Err(ConversionError(
            "NULL is only supported for top-level CQL bind markers and tuple or UDT fields"
                .to_string(),
        ));
    }
    let converted = match column_type {
        ColumnType::Native(native) => match native {
            NativeType::Ascii => CqlValue::Ascii(as_string(value)),
            NativeType::Text => CqlValue::Text(as_string(value)),
            NativeType::Boolean => CqlValue::Boolean(parse_value(value, "boolean")?),
            NativeType::Blob => match value {
                Value::Bytes(bytes) => CqlValue::Blob(bytes.clone()),
                _ => return type_error(value, column_type),
            },
            NativeType::Counter => CqlValue::Counter(Counter(parse_value(value, "counter")?)),
            NativeType::Decimal => {
                let decimal = BigDecimal::from_str(&as_string(value))
                    .map_err(|error| ConversionError(error.to_string()))?;
                CqlValue::Decimal(
                    CqlDecimal::try_from(decimal)
                        .map_err(|error| ConversionError(error.to_string()))?,
                )
            }
            NativeType::Date => {
                let date = NaiveDate::parse_from_str(&as_string(value), "%Y-%m-%d")
                    .map_err(|error| ConversionError(error.to_string()))?;
                CqlValue::Date(CqlDate::from(date))
            }
            NativeType::Double => CqlValue::Double(parse_value(value, "double")?),
            NativeType::Duration => CqlValue::Duration(parse_duration(&as_string(value))?),
            NativeType::Float => CqlValue::Float(parse_value(value, "float")?),
            NativeType::Int => CqlValue::Int(parse_value(value, "int")?),
            NativeType::BigInt => CqlValue::BigInt(parse_value(value, "bigint")?),
            NativeType::Timestamp => {
                let datetime = parse_datetime(&as_string(value))?;
                CqlValue::Timestamp(CqlTimestamp::from(datetime.and_utc()))
            }
            NativeType::Inet => CqlValue::Inet(
                as_string(value)
                    .parse()
                    .map_err(|error| ConversionError(format!("invalid inet address: {error}")))?,
            ),
            NativeType::SmallInt => CqlValue::SmallInt(parse_value(value, "smallint")?),
            NativeType::TinyInt => CqlValue::TinyInt(parse_value(value, "tinyint")?),
            NativeType::Time => {
                let time = NaiveTime::parse_from_str(&as_string(value), "%H:%M:%S%.f")
                    .map_err(|error| ConversionError(error.to_string()))?;
                CqlValue::Time(
                    CqlTime::try_from(time).map_err(|error| ConversionError(error.to_string()))?,
                )
            }
            NativeType::Timeuuid => {
                let uuid = parse_uuid(value)?;
                CqlValue::Timeuuid(uuid.into())
            }
            NativeType::Uuid => CqlValue::Uuid(parse_uuid(value)?),
            NativeType::Varint => {
                let integer = BigInt::from_str(&as_string(value))
                    .map_err(|error| ConversionError(error.to_string()))?;
                CqlValue::Varint(CqlVarint::from(integer))
            }
            _ => return type_error(value, column_type),
        },
        ColumnType::Collection { typ, .. } => match typ {
            CollectionType::List(element_type) => {
                CqlValue::List(convert_array(value, element_type, column_type)?)
            }
            CollectionType::Set(element_type) => {
                CqlValue::Set(convert_array(value, element_type, column_type)?)
            }
            CollectionType::Map(key_type, value_type) => {
                let Value::Map(map) = value else {
                    return type_error(value, column_type);
                };
                let entries = map
                    .iter()
                    .map(|(key, value)| {
                        Ok((
                            value_to_cql(key, key_type)?,
                            value_to_cql(value, value_type)?,
                        ))
                    })
                    .collect::<Result<Vec<_>>>()?;
                CqlValue::Map(entries)
            }
            _ => return type_error(value, column_type),
        },
        ColumnType::Vector { typ, dimensions } => {
            let values = convert_array(value, typ, column_type)?;
            if values.len() != usize::from(*dimensions) {
                return Err(ConversionError(format!(
                    "CQL vector requires {dimensions} values, received {}",
                    values.len()
                )));
            }
            CqlValue::Vector(values)
        }
        ColumnType::Tuple(types) => {
            let Value::Array(values) = value else {
                return type_error(value, column_type);
            };
            if values.len() != types.len() {
                return Err(ConversionError(format!(
                    "CQL tuple requires {} values, received {}",
                    types.len(),
                    values.len()
                )));
            }
            let values = values
                .iter()
                .zip(types)
                .map(|(value, typ)| {
                    if value.is_null() {
                        Ok(None)
                    } else {
                        value_to_cql(value, typ).map(Some)
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            CqlValue::Tuple(values)
        }
        ColumnType::UserDefinedType { definition, .. } => {
            let Value::Map(map) = value else {
                return type_error(value, column_type);
            };
            let fields = definition
                .field_types
                .iter()
                .map(|(field_name, field_type)| {
                    let key = Value::String(field_name.to_string());
                    let value = map.get(&key);
                    let value = match value {
                        None | Some(Value::Null) => None,
                        Some(value) => Some(value_to_cql(value, field_type)?),
                    };
                    Ok((field_name.to_string(), value))
                })
                .collect::<Result<Vec<_>>>()?;
            CqlValue::UserDefinedType {
                keyspace: definition.keyspace.to_string(),
                name: definition.name.to_string(),
                fields,
            }
        }
        _ => return type_error(value, column_type),
    };
    Ok(converted)
}

fn convert_array(
    value: &Value,
    element_type: &ColumnType<'_>,
    parent_type: &ColumnType<'_>,
) -> Result<Vec<CqlValue>> {
    let Value::Array(values) = value else {
        return type_error(value, parent_type);
    };
    values
        .iter()
        .map(|value| value_to_cql(value, element_type))
        .collect()
}

fn as_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn parse_value<T>(value: &Value, type_name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    as_string(value).parse().map_err(|error| {
        ConversionError(format!(
            "cannot convert {value} to CQL {type_name}: {error}"
        ))
    })
}

fn parse_uuid(value: &Value) -> Result<uuid::Uuid> {
    match value {
        Value::Uuid(value) => Ok(*value),
        _ => uuid::Uuid::parse_str(&as_string(value))
            .map_err(|error| ConversionError(error.to_string())),
    }
}

fn parse_datetime(value: &str) -> Result<NaiveDateTime> {
    ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"]
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
        .ok_or_else(|| ConversionError(format!("invalid CQL timestamp: {value}")))
}

fn parse_duration(value: &str) -> Result<CqlDuration> {
    let mut parts = value.split(',').map(str::trim);
    let months = parse_duration_part(parts.next(), "months")?;
    let days = parse_duration_part(parts.next(), "days")?;
    let nanoseconds = parse_duration_part(parts.next(), "nanoseconds")?;
    if parts.next().is_some() {
        return Err(ConversionError(format!("invalid CQL duration: {value}")));
    }
    Ok(CqlDuration {
        months,
        days,
        nanoseconds,
    })
}

fn parse_duration_part<T>(part: Option<&str>, name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let part = part.ok_or_else(|| {
        ConversionError("CQL duration must use months=<n>, days=<n>, nanoseconds=<n>".to_string())
    })?;
    let (actual_name, value) = part
        .split_once('=')
        .ok_or_else(|| ConversionError(format!("invalid CQL duration component: {part}")))?;
    if actual_name != name {
        return Err(ConversionError(format!(
            "expected CQL duration component {name}, found {actual_name}"
        )));
    }
    value
        .parse()
        .map_err(|error| ConversionError(format!("invalid CQL duration {name}: {error}")))
}

fn type_error<T>(value: &Value, column_type: &ColumnType<'_>) -> Result<T> {
    Err(ConversionError(format!(
        "cannot convert {value:?} to CQL type {}",
        type_name(column_type)
    )))
}

pub(crate) fn cql_to_value(value: CqlValue) -> Value {
    match value {
        CqlValue::Ascii(value) | CqlValue::Text(value) => Value::String(value),
        CqlValue::Boolean(value) => Value::Bool(value),
        CqlValue::Blob(value) => Value::Bytes(value),
        CqlValue::Counter(Counter(value)) | CqlValue::BigInt(value) => Value::I64(value),
        CqlValue::Decimal(value) => {
            let value = BigDecimal::from(value).to_string();
            Decimal::from_str(&value).map_or(Value::String(value), Value::Decimal)
        }
        CqlValue::Date(value) => {
            let date: std::result::Result<NaiveDate, _> = value.try_into();
            date.ok()
                .and_then(|date| date.to_string().parse().ok())
                .map_or_else(|| Value::String(value.0.to_string()), Value::Date)
        }
        CqlValue::Double(value) => Value::F64(value),
        CqlValue::Duration(value) => Value::String(format!(
            "months={}, days={}, nanoseconds={}",
            value.months, value.days, value.nanoseconds
        )),
        CqlValue::Empty => Value::String(String::new()),
        CqlValue::Float(value) => Value::F32(value),
        CqlValue::Int(value) => Value::I32(value),
        CqlValue::Timestamp(value) => {
            let timestamp: std::result::Result<chrono::DateTime<chrono::Utc>, _> = value.try_into();
            timestamp
                .ok()
                .and_then(|timestamp| {
                    timestamp
                        .naive_utc()
                        .format("%Y-%m-%dT%H:%M:%S%.f")
                        .to_string()
                        .parse()
                        .ok()
                })
                .map_or_else(|| Value::String(value.0.to_string()), Value::DateTime)
        }
        CqlValue::Inet(value) => Value::String(value.to_string()),
        CqlValue::List(values) | CqlValue::Set(values) | CqlValue::Vector(values) => {
            Value::Array(values.into_iter().map(cql_to_value).collect())
        }
        CqlValue::Map(values) => Value::Map(
            values
                .into_iter()
                .map(|(key, value)| (cql_to_value(key), cql_to_value(value)))
                .collect(),
        ),
        CqlValue::UserDefinedType { fields, .. } => Value::Map(
            fields
                .into_iter()
                .map(|(name, value)| (Value::String(name), value.map_or(Value::Null, cql_to_value)))
                .collect(),
        ),
        CqlValue::SmallInt(value) => Value::I16(value),
        CqlValue::TinyInt(value) => Value::I8(value),
        CqlValue::Time(value) => {
            let time: std::result::Result<NaiveTime, _> = value.try_into();
            time.ok()
                .and_then(|time| time.to_string().parse().ok())
                .map_or_else(|| Value::String(value.0.to_string()), Value::Time)
        }
        CqlValue::Timeuuid(value) => Value::Uuid(value.into()),
        CqlValue::Tuple(values) => Value::Array(
            values
                .into_iter()
                .map(|value| value.map_or(Value::Null, cql_to_value))
                .collect(),
        ),
        CqlValue::Uuid(value) => Value::Uuid(value),
        CqlValue::Varint(value) => {
            let value = BigInt::from(value).to_string();
            value
                .parse::<i128>()
                .map_or(Value::String(value), Value::I128)
        }
        other => Value::String(format!("{other:?}")),
    }
}

pub(crate) fn type_name(column_type: &ColumnType<'_>) -> String {
    match column_type {
        ColumnType::Native(native) => match native {
            NativeType::Ascii => "ascii",
            NativeType::Boolean => "boolean",
            NativeType::Blob => "blob",
            NativeType::Counter => "counter",
            NativeType::Date => "date",
            NativeType::Decimal => "decimal",
            NativeType::Double => "double",
            NativeType::Duration => "duration",
            NativeType::Float => "float",
            NativeType::Int => "int",
            NativeType::BigInt => "bigint",
            NativeType::Text => "text",
            NativeType::Timestamp => "timestamp",
            NativeType::Inet => "inet",
            NativeType::SmallInt => "smallint",
            NativeType::TinyInt => "tinyint",
            NativeType::Time => "time",
            NativeType::Timeuuid => "timeuuid",
            NativeType::Uuid => "uuid",
            NativeType::Varint => "varint",
            _ => "unknown",
        }
        .to_string(),
        ColumnType::Collection { frozen, typ } => {
            let inner = match typ {
                CollectionType::List(element) => format!("list<{}>", type_name(element)),
                CollectionType::Map(key, value) => {
                    format!("map<{}, {}>", type_name(key), type_name(value))
                }
                CollectionType::Set(element) => format!("set<{}>", type_name(element)),
                _ => "unknown".to_string(),
            };
            frozen_type(*frozen, inner)
        }
        ColumnType::Vector { typ, dimensions } => {
            format!("vector<{}, {dimensions}>", type_name(typ))
        }
        ColumnType::UserDefinedType { frozen, definition } => frozen_type(
            *frozen,
            format!("{}.{}", definition.keyspace, definition.name),
        ),
        ColumnType::Tuple(types) => format!(
            "tuple<{}>",
            types.iter().map(type_name).collect::<Vec<_>>().join(", ")
        ),
        _ => "unknown".to_string(),
    }
}

fn frozen_type(frozen: bool, inner: String) -> String {
    if frozen {
        format!("frozen<{inner}>")
    } else {
        inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scylla::frame::response::result::{CollectionType, UserDefinedType};
    use std::borrow::Cow;
    use std::sync::Arc;

    #[test]
    fn converts_bind_values_and_rejects_bad_counts() -> Result<()> {
        let bigint = ColumnType::Native(NativeType::BigInt);
        let text = ColumnType::Native(NativeType::Text);
        assert_eq!(
            values_to_cql(
                &[Value::Null, Value::String("hello".to_string())],
                &[&bigint, &text]
            )?,
            vec![None, Some(CqlValue::Text("hello".to_string()))]
        );
        assert!(values_to_cql(&[Value::I64(1)], &[&bigint, &text]).is_err());
        assert!(value_to_cql(&Value::Null, &text).is_err());
        assert!(value_to_cql(&Value::String("no".to_string()), &bigint).is_err());
        Ok(())
    }

    #[test]
    fn converts_native_values() -> Result<()> {
        assert_eq!(
            value_to_cql(&Value::I64(42), &ColumnType::Native(NativeType::BigInt))?,
            CqlValue::BigInt(42)
        );
        assert_eq!(
            value_to_cql(&Value::I32(7), &ColumnType::Native(NativeType::Ascii))?,
            CqlValue::Ascii("7".to_string())
        );
        assert_eq!(
            value_to_cql(&Value::Bool(true), &ColumnType::Native(NativeType::Boolean))?,
            CqlValue::Boolean(true)
        );
        assert_eq!(
            value_to_cql(
                &Value::Bytes(vec![1, 2]),
                &ColumnType::Native(NativeType::Blob)
            )?,
            CqlValue::Blob(vec![1, 2])
        );
        assert_eq!(
            value_to_cql(&Value::I64(8), &ColumnType::Native(NativeType::Counter))?,
            CqlValue::Counter(Counter(8))
        );
        assert!(matches!(
            value_to_cql(
                &Value::String("12.50".to_string()),
                &ColumnType::Native(NativeType::Decimal)
            )?,
            CqlValue::Decimal(_)
        ));
        assert!(matches!(
            value_to_cql(
                &Value::String("2024-01-02".to_string()),
                &ColumnType::Native(NativeType::Date)
            )?,
            CqlValue::Date(_)
        ));
        assert_eq!(
            value_to_cql(&Value::F64(1.5), &ColumnType::Native(NativeType::Double))?,
            CqlValue::Double(1.5)
        );
        assert_eq!(
            value_to_cql(
                &Value::String("months=1, days=2, nanoseconds=3".to_string()),
                &ColumnType::Native(NativeType::Duration)
            )?,
            CqlValue::Duration(CqlDuration {
                months: 1,
                days: 2,
                nanoseconds: 3,
            })
        );
        assert_eq!(
            value_to_cql(&Value::F32(2.5), &ColumnType::Native(NativeType::Float))?,
            CqlValue::Float(2.5)
        );
        assert_eq!(
            value_to_cql(&Value::I32(9), &ColumnType::Native(NativeType::Int))?,
            CqlValue::Int(9)
        );
        Ok(())
    }

    #[test]
    fn converts_native_temporal_and_identifier_values() -> Result<()> {
        let uuid = uuid::Uuid::from_u128(0x67e5_501c_a4aa_4262_8fb4_62e2_5b24_7b4f);
        assert!(matches!(
            value_to_cql(
                &Value::String("2024-01-02 03:04:05.006".to_string()),
                &ColumnType::Native(NativeType::Timestamp)
            )?,
            CqlValue::Timestamp(_)
        ));
        assert!(matches!(
            value_to_cql(
                &Value::String("127.0.0.1".to_string()),
                &ColumnType::Native(NativeType::Inet)
            )?,
            CqlValue::Inet(_)
        ));
        assert_eq!(
            value_to_cql(&Value::I16(10), &ColumnType::Native(NativeType::SmallInt))?,
            CqlValue::SmallInt(10)
        );
        assert_eq!(
            value_to_cql(&Value::I8(11), &ColumnType::Native(NativeType::TinyInt))?,
            CqlValue::TinyInt(11)
        );
        assert!(matches!(
            value_to_cql(
                &Value::String("03:04:05.006".to_string()),
                &ColumnType::Native(NativeType::Time)
            )?,
            CqlValue::Time(_)
        ));
        assert_eq!(
            value_to_cql(&Value::Uuid(uuid), &ColumnType::Native(NativeType::Uuid))?,
            CqlValue::Uuid(uuid)
        );
        assert_eq!(
            value_to_cql(
                &Value::String(uuid.to_string()),
                &ColumnType::Native(NativeType::Timeuuid)
            )?,
            CqlValue::Timeuuid(uuid.into())
        );
        assert!(matches!(
            value_to_cql(&Value::I128(12), &ColumnType::Native(NativeType::Varint))?,
            CqlValue::Varint(_)
        ));
        Ok(())
    }

    #[test]
    fn converts_compound_values() -> Result<()> {
        let list_type = ColumnType::Collection {
            frozen: false,
            typ: CollectionType::List(Box::new(ColumnType::Native(NativeType::Text))),
        };
        assert_eq!(
            value_to_cql(
                &Value::Array(vec![Value::String("a".to_string())]),
                &list_type
            )?,
            CqlValue::List(vec![CqlValue::Text("a".to_string())])
        );
        let set_type = ColumnType::Collection {
            frozen: false,
            typ: CollectionType::Set(Box::new(ColumnType::Native(NativeType::Int))),
        };
        assert_eq!(
            value_to_cql(&Value::Array(vec![Value::I32(1)]), &set_type)?,
            CqlValue::Set(vec![CqlValue::Int(1)])
        );
        let map_type = ColumnType::Collection {
            frozen: false,
            typ: CollectionType::Map(
                Box::new(ColumnType::Native(NativeType::Text)),
                Box::new(ColumnType::Native(NativeType::Int)),
            ),
        };
        let map_value = Value::Map(
            [(Value::String("one".to_string()), Value::I32(1))]
                .into_iter()
                .collect(),
        );
        assert_eq!(
            value_to_cql(&map_value, &map_type)?,
            CqlValue::Map(vec![(CqlValue::Text("one".to_string()), CqlValue::Int(1))])
        );

        let vector_type = ColumnType::Vector {
            typ: Box::new(ColumnType::Native(NativeType::Float)),
            dimensions: 2,
        };
        assert_eq!(
            value_to_cql(
                &Value::Array(vec![Value::F32(1.0), Value::F32(2.0)]),
                &vector_type
            )?,
            CqlValue::Vector(vec![CqlValue::Float(1.0), CqlValue::Float(2.0)])
        );
        let tuple_type = ColumnType::Tuple(vec![
            ColumnType::Native(NativeType::Int),
            ColumnType::Native(NativeType::Text),
        ]);
        assert_eq!(
            value_to_cql(&Value::Array(vec![Value::I32(1), Value::Null]), &tuple_type)?,
            CqlValue::Tuple(vec![Some(CqlValue::Int(1)), None])
        );

        let udt_type = ColumnType::UserDefinedType {
            frozen: true,
            definition: Arc::new(UserDefinedType {
                name: Cow::Borrowed("address"),
                keyspace: Cow::Borrowed("app"),
                field_types: vec![
                    (
                        Cow::Borrowed("street"),
                        ColumnType::Native(NativeType::Text),
                    ),
                    (Cow::Borrowed("zip"), ColumnType::Native(NativeType::Int)),
                ],
            }),
        };
        let udt_value = Value::Map(
            [(
                Value::String("street".to_string()),
                Value::String("Main".to_string()),
            )]
            .into_iter()
            .collect(),
        );
        assert_eq!(
            value_to_cql(&udt_value, &udt_type)?,
            CqlValue::UserDefinedType {
                keyspace: "app".to_string(),
                name: "address".to_string(),
                fields: vec![
                    (
                        "street".to_string(),
                        Some(CqlValue::Text("Main".to_string()))
                    ),
                    ("zip".to_string(), None),
                ],
            }
        );
        Ok(())
    }

    #[test]
    fn rejects_invalid_values() {
        let native_errors = [
            NativeType::Boolean,
            NativeType::Decimal,
            NativeType::Date,
            NativeType::Double,
            NativeType::Float,
            NativeType::Int,
            NativeType::BigInt,
            NativeType::Timestamp,
            NativeType::Inet,
            NativeType::SmallInt,
            NativeType::TinyInt,
            NativeType::Time,
            NativeType::Timeuuid,
            NativeType::Uuid,
            NativeType::Varint,
        ];
        for native_type in native_errors {
            assert!(
                value_to_cql(
                    &Value::String("not-a-value".to_string()),
                    &ColumnType::Native(native_type)
                )
                .is_err()
            );
        }
        assert!(
            value_to_cql(
                &Value::String("bytes".to_string()),
                &ColumnType::Native(NativeType::Blob)
            )
            .is_err()
        );
        for duration in [
            "months=1, days=2",
            "months=1, days=2, nanoseconds=3, extra=4",
            "month=1, days=2, nanoseconds=3",
            "months, days=2, nanoseconds=3",
            "months=x, days=2, nanoseconds=3",
        ] {
            assert!(parse_duration(duration).is_err());
        }

        let list_type = ColumnType::Collection {
            frozen: false,
            typ: CollectionType::List(Box::new(ColumnType::Native(NativeType::Text))),
        };
        let map_type = ColumnType::Collection {
            frozen: false,
            typ: CollectionType::Map(
                Box::new(ColumnType::Native(NativeType::Text)),
                Box::new(ColumnType::Native(NativeType::Int)),
            ),
        };
        let vector_type = ColumnType::Vector {
            typ: Box::new(ColumnType::Native(NativeType::Int)),
            dimensions: 2,
        };
        let tuple_type = ColumnType::Tuple(vec![ColumnType::Native(NativeType::Int)]);
        assert!(value_to_cql(&Value::I32(1), &list_type).is_err());
        assert!(value_to_cql(&Value::I32(1), &map_type).is_err());
        assert!(value_to_cql(&Value::Array(vec![Value::I32(1)]), &vector_type).is_err());
        assert!(value_to_cql(&Value::I32(1), &tuple_type).is_err());
        assert!(value_to_cql(&Value::Array(Vec::new()), &tuple_type).is_err());
    }

    #[test]
    fn converts_cql_results() -> Result<()> {
        let date = NaiveDate::parse_from_str("2024-01-02", "%Y-%m-%d")
            .map_err(|error| ConversionError(error.to_string()))?;
        let datetime = parse_datetime("2024-01-02T03:04:05.006")?;
        let decimal =
            BigDecimal::from_str("12.50").map_err(|error| ConversionError(error.to_string()))?;
        let cql_decimal =
            CqlDecimal::try_from(decimal).map_err(|error| ConversionError(error.to_string()))?;

        assert_eq!(
            cql_to_value(CqlValue::Ascii("a".to_string())),
            Value::String("a".to_string())
        );
        assert_eq!(cql_to_value(CqlValue::Boolean(true)), Value::Bool(true));
        assert_eq!(
            cql_to_value(CqlValue::Blob(vec![1, 2])),
            Value::Bytes(vec![1, 2])
        );
        assert_eq!(cql_to_value(CqlValue::Counter(Counter(3))), Value::I64(3));
        assert!(matches!(
            cql_to_value(CqlValue::Decimal(cql_decimal)),
            Value::Decimal(_)
        ));
        assert!(matches!(
            cql_to_value(CqlValue::Date(CqlDate::from(date))),
            Value::Date(_)
        ));
        assert_eq!(cql_to_value(CqlValue::Double(1.5)), Value::F64(1.5));
        assert_eq!(
            cql_to_value(CqlValue::Duration(CqlDuration {
                months: 1,
                days: 2,
                nanoseconds: 3,
            })),
            Value::String("months=1, days=2, nanoseconds=3".to_string())
        );
        assert_eq!(cql_to_value(CqlValue::Empty), Value::String(String::new()));
        assert_eq!(cql_to_value(CqlValue::Float(2.5)), Value::F32(2.5));
        assert_eq!(cql_to_value(CqlValue::Int(4)), Value::I32(4));
        assert!(matches!(
            cql_to_value(CqlValue::Timestamp(CqlTimestamp::from(datetime.and_utc()))),
            Value::DateTime(_)
        ));
        assert_eq!(
            cql_to_value(CqlValue::Inet("127.0.0.1".parse().map_err(|error| {
                ConversionError(format!("invalid IP address: {error}"))
            })?)),
            Value::String("127.0.0.1".to_string())
        );
        assert_eq!(
            cql_to_value(CqlValue::List(vec![CqlValue::Boolean(true)])),
            Value::Array(vec![Value::Bool(true)])
        );
        assert_eq!(
            cql_to_value(CqlValue::Set(vec![CqlValue::Int(5)])),
            Value::Array(vec![Value::I32(5)])
        );
        assert_eq!(
            cql_to_value(CqlValue::Vector(vec![CqlValue::Float(1.0)])),
            Value::Array(vec![Value::F32(1.0)])
        );
        assert_eq!(
            cql_to_value(CqlValue::Map(vec![(
                CqlValue::Text("a".to_string()),
                CqlValue::Int(1)
            )])),
            Value::Map(
                [(Value::String("a".to_string()), Value::I32(1))]
                    .into_iter()
                    .collect()
            )
        );
        Ok(())
    }

    #[test]
    fn converts_complex_cql_results() -> Result<()> {
        let uuid = uuid::Uuid::from_u128(0x67e5_501c_a4aa_4262_8fb4_62e2_5b24_7b4f);
        let time = NaiveTime::parse_from_str("03:04:05.006", "%H:%M:%S%.f")
            .map_err(|error| ConversionError(error.to_string()))?;
        assert_eq!(cql_to_value(CqlValue::SmallInt(6)), Value::I16(6));
        assert_eq!(cql_to_value(CqlValue::TinyInt(7)), Value::I8(7));
        assert!(matches!(
            cql_to_value(CqlValue::Time(
                CqlTime::try_from(time).map_err(|error| ConversionError(error.to_string()))?
            )),
            Value::Time(_)
        ));
        assert_eq!(
            cql_to_value(CqlValue::Timeuuid(uuid.into())),
            Value::Uuid(uuid)
        );
        assert_eq!(cql_to_value(CqlValue::Uuid(uuid)), Value::Uuid(uuid));
        assert_eq!(
            cql_to_value(CqlValue::Tuple(vec![Some(CqlValue::Int(8)), None])),
            Value::Array(vec![Value::I32(8), Value::Null])
        );
        assert_eq!(
            cql_to_value(CqlValue::UserDefinedType {
                keyspace: "app".to_string(),
                name: "address".to_string(),
                fields: vec![("street".to_string(), None)],
            }),
            Value::Map(
                [(Value::String("street".to_string()), Value::Null)]
                    .into_iter()
                    .collect()
            )
        );
        assert_eq!(
            cql_to_value(CqlValue::Varint(CqlVarint::from(BigInt::from(9)))),
            Value::I128(9)
        );
        let large = BigInt::from(i128::MAX) + BigInt::from(1_u8);
        assert!(matches!(
            cql_to_value(CqlValue::Varint(CqlVarint::from(large))),
            Value::String(_)
        ));
        assert!(matches!(
            cql_to_value(CqlValue::Date(CqlDate(u32::MAX))),
            Value::String(_)
        ));
        assert!(matches!(
            cql_to_value(CqlValue::Timestamp(CqlTimestamp(i64::MAX))),
            Value::String(_)
        ));
        assert!(matches!(
            cql_to_value(CqlValue::Time(CqlTime(i64::MAX))),
            Value::String(_)
        ));
        Ok(())
    }

    #[test]
    fn renders_cql_types() {
        let native_types = [
            (NativeType::Ascii, "ascii"),
            (NativeType::Boolean, "boolean"),
            (NativeType::Blob, "blob"),
            (NativeType::Counter, "counter"),
            (NativeType::Date, "date"),
            (NativeType::Decimal, "decimal"),
            (NativeType::Double, "double"),
            (NativeType::Duration, "duration"),
            (NativeType::Float, "float"),
            (NativeType::Int, "int"),
            (NativeType::BigInt, "bigint"),
            (NativeType::Text, "text"),
            (NativeType::Timestamp, "timestamp"),
            (NativeType::Inet, "inet"),
            (NativeType::SmallInt, "smallint"),
            (NativeType::TinyInt, "tinyint"),
            (NativeType::Time, "time"),
            (NativeType::Timeuuid, "timeuuid"),
            (NativeType::Uuid, "uuid"),
            (NativeType::Varint, "varint"),
        ];
        for (native_type, name) in native_types {
            assert_eq!(type_name(&ColumnType::Native(native_type)), name);
        }

        let list = ColumnType::Collection {
            frozen: false,
            typ: CollectionType::List(Box::new(ColumnType::Native(NativeType::Text))),
        };
        let map = ColumnType::Collection {
            frozen: true,
            typ: CollectionType::Map(
                Box::new(ColumnType::Native(NativeType::Text)),
                Box::new(ColumnType::Native(NativeType::Int)),
            ),
        };
        let set = ColumnType::Collection {
            frozen: false,
            typ: CollectionType::Set(Box::new(ColumnType::Native(NativeType::Uuid))),
        };
        assert_eq!(type_name(&list), "list<text>");
        assert_eq!(type_name(&map), "frozen<map<text, int>>");
        assert_eq!(type_name(&set), "set<uuid>");
        assert_eq!(
            type_name(&ColumnType::Vector {
                typ: Box::new(ColumnType::Native(NativeType::Float)),
                dimensions: 3,
            }),
            "vector<float, 3>"
        );
        assert_eq!(
            type_name(&ColumnType::Tuple(vec![
                ColumnType::Native(NativeType::Int),
                ColumnType::Native(NativeType::Text),
            ])),
            "tuple<int, text>"
        );
        let udt = ColumnType::UserDefinedType {
            frozen: true,
            definition: Arc::new(UserDefinedType {
                name: Cow::Borrowed("address"),
                keyspace: Cow::Borrowed("app"),
                field_types: Vec::new(),
            }),
        };
        assert_eq!(type_name(&udt), "frozen<app.address>");
    }
}
