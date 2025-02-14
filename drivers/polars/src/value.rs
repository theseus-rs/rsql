use chrono::{NaiveDate, NaiveTime, TimeDelta};
use indexmap::IndexMap;
use polars::datatypes::AnyValue;
use rsql_driver::Value;
use std::ops::Add;

pub trait ToValue {
    fn to_value(&self) -> Value;
}

impl ToValue for AnyValue<'_> {
    fn to_value(&self) -> Value {
        match self {
            AnyValue::Null => Value::Null,
            AnyValue::Boolean(v) => Value::Bool(*v),
            AnyValue::Binary(v) => Value::Bytes(v.to_vec()),
            AnyValue::BinaryOwned(v) => Value::Bytes(v.clone()),
            AnyValue::Date(days) => {
                let default_date = NaiveDate::default();
                let date = default_date.add(TimeDelta::days(i64::from(*days)));
                Value::Date(date)
            }
            AnyValue::Float32(v) => Value::F32(*v),
            AnyValue::Float64(v) => Value::F64(*v),
            AnyValue::Int8(v) => Value::I8(*v),
            AnyValue::Int16(v) => Value::I16(*v),
            AnyValue::Int32(v) => Value::I32(*v),
            AnyValue::Int64(v) => Value::I64(*v),
            AnyValue::List(series) => {
                let mut values = Vec::new();
                for value in series.iter() {
                    values.push(value.to_value());
                }
                Value::Array(values)
            }
            AnyValue::String(v) => Value::String((*v).to_string()),
            any_value @ AnyValue::Struct(_, _, fields) => {
                let mut values = vec![];
                any_value._materialize_struct_av(&mut values);
                let mut map = IndexMap::new();
                for (field, value) in fields.iter().zip(values.iter()) {
                    let field_name = Value::String(field.name().to_string());
                    let value = value.to_value();
                    map.insert(field_name, value);
                }
                Value::Map(map)
            }
            AnyValue::StructOwned(tuple) => {
                let values = &tuple.0;
                let fields = &tuple.1;
                let mut map = IndexMap::new();
                for (field, value) in fields.iter().zip(values.iter()) {
                    let field_name = Value::String(field.name().to_string());
                    let value = value.to_value();
                    map.insert(field_name, value);
                }
                Value::Map(map)
            }
            AnyValue::Time(nanos) => {
                let seconds = u32::try_from(nanos / 1_000_000_000).unwrap_or(0);
                let nanoseconds = u32::try_from(nanos % 1_000_000_000).unwrap_or(0);
                if let Some(time) =
                    NaiveTime::from_num_seconds_from_midnight_opt(seconds, nanoseconds)
                {
                    Value::Time(time)
                } else {
                    Value::Null
                }
            }
            AnyValue::UInt8(v) => Value::U8(*v),
            AnyValue::UInt16(v) => Value::U16(*v),
            AnyValue::UInt32(v) => Value::U32(*v),
            AnyValue::UInt64(v) => Value::U64(*v),
            _ => Value::String(self.to_string()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use polars::datatypes::{DataType, Field, PlSmallStr};
    use polars::prelude::NamedFrom;
    use polars::series::Series;

    #[test]
    fn test_null() {
        let any_value = AnyValue::Null;
        let value = any_value.to_value();
        assert_eq!(Value::Null, value);
    }

    #[test]
    fn test_boolean() {
        let any_value = AnyValue::Boolean(true);
        let value = any_value.to_value();
        assert_eq!(Value::Bool(true), value);
    }

    #[test]
    fn test_binary() {
        let any_value = AnyValue::Binary(&[0x01, 0x02, 0x03]);
        let value = any_value.to_value();
        assert_eq!(Value::Bytes(vec![0x01, 0x02, 0x03]), value);
    }

    #[test]
    fn test_binary_owned() {
        let any_value = AnyValue::BinaryOwned(vec![0x01, 0x02, 0x03]);
        let value = any_value.to_value();
        assert_eq!(Value::Bytes(vec![0x01, 0x02, 0x03]), value);
    }

    #[test]
    fn test_date() {
        let any_value = AnyValue::Date(18628);
        let value = any_value.to_value();
        let expected = NaiveDate::from_ymd_opt(2021, 1, 1).expect("Invalid date");
        assert_eq!(Value::Date(expected), value);
    }

    #[test]
    fn test_float32() {
        let pi = std::f32::consts::PI;
        let any_value = AnyValue::Float32(pi);
        let value = any_value.to_value();
        assert_eq!(Value::F32(pi), value);
    }

    #[test]
    fn test_float64() {
        let pi = std::f64::consts::PI;
        let any_value = AnyValue::Float64(pi);
        let value = any_value.to_value();
        assert_eq!(Value::F64(pi), value);
    }

    #[test]
    fn test_list() {
        let series = Series::new("int series".into(), &[1, 2]);
        let any_value = AnyValue::List(series);
        let value = any_value.to_value();
        assert_eq!(Value::Array(vec![Value::I32(1), Value::I32(2)]), value);
    }

    #[test]
    fn test_int8() {
        let any_value = AnyValue::Int8(8);
        let value = any_value.to_value();
        assert_eq!(Value::I8(8), value);
    }

    #[test]
    fn test_int16() {
        let any_value = AnyValue::Int16(16);
        let value = any_value.to_value();
        assert_eq!(Value::I16(16), value);
    }

    #[test]
    fn test_int32() {
        let any_value = AnyValue::Int32(32);
        let value = any_value.to_value();
        assert_eq!(Value::I32(32), value);
    }

    #[test]
    fn test_int64() {
        let any_value = AnyValue::Int64(64);
        let value = any_value.to_value();
        assert_eq!(Value::I64(64), value);
    }

    #[test]
    fn test_string() {
        let any_value = AnyValue::String("test");
        let value = any_value.to_value();
        assert_eq!(Value::String("test".to_string()), value);
    }

    #[test]
    fn test_struct_owned() {
        let fields = vec![
            Field::new(PlSmallStr::from("field1"), DataType::Int32),
            Field::new(PlSmallStr::from("field2"), DataType::String),
        ];
        let values = vec![AnyValue::Int32(42), AnyValue::String("example")];
        let any_value = AnyValue::StructOwned(Box::new((values, fields)));
        let value = any_value.to_value();
        let mut map = IndexMap::new();
        map.insert(Value::String("field1".to_string()), Value::I32(42));
        map.insert(
            Value::String("field2".to_string()),
            Value::String("example".to_string()),
        );
        let expected = Value::Map(map);
        assert_eq!(expected, value);
    }

    #[test]
    fn test_time() {
        let hours = 3;
        let minutes = 25;
        let seconds = 45;
        let nanos = 678_901;
        let nanoseconds = ((hours * 3600 + minutes * 60 + seconds) * 1_000_000_000) + nanos;
        let any_value = AnyValue::Time(nanoseconds);
        let value = any_value.to_value();
        let expected = NaiveTime::from_hms_nano_opt(3, 25, 45, 678_901).expect("Invalid time");
        assert_eq!(Value::Time(expected), value);
    }

    #[test]
    fn test_uint8() {
        let any_value = AnyValue::UInt8(8);
        let value = any_value.to_value();
        assert_eq!(Value::U8(8), value);
    }

    #[test]
    fn test_uint16() {
        let any_value = AnyValue::UInt16(16);
        let value = any_value.to_value();
        assert_eq!(Value::U16(16), value);
    }

    #[test]
    fn test_uint32() {
        let any_value = AnyValue::UInt32(32);
        let value = any_value.to_value();
        assert_eq!(Value::U32(32), value);
    }

    #[test]
    fn test_uint64() {
        let any_value = AnyValue::UInt64(64);
        let value = any_value.to_value();
        assert_eq!(Value::U64(64), value);
    }
}
