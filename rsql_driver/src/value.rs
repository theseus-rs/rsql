use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use indexmap::IndexMap;
use num_format::{Locale, ToFormattedString};
use serde::{Serialize, Serializer};
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Bytes(Vec<u8>),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    String(String),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
    DateTime(chrono::NaiveDateTime),
    Uuid(uuid::Uuid),
    Json(serde_json::Value),
    Array(Vec<Value>),
    Map(IndexMap<Value, Value>),
}

impl Value {
    #[must_use]
    pub fn to_formatted_string(&self, locale: &Locale) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::Bool(value) => value.to_string(),
            Value::Bytes(bytes) => STANDARD.encode(bytes),
            Value::I8(value) => value.to_formatted_string(locale),
            Value::I16(value) => value.to_formatted_string(locale),
            Value::I32(value) => value.to_formatted_string(locale),
            Value::I64(value) => value.to_formatted_string(locale),
            Value::I128(value) => value.to_formatted_string(locale),
            Value::U8(value) => value.to_formatted_string(locale),
            Value::U16(value) => value.to_formatted_string(locale),
            Value::U32(value) => value.to_formatted_string(locale),
            Value::U64(value) => value.to_formatted_string(locale),
            Value::U128(value) => value.to_formatted_string(locale),
            Value::F32(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::String(value) => value.to_string(),
            Value::Date(value) => value.to_string(),
            Value::Time(value) => value.to_string(),
            Value::DateTime(value) => value.to_string(),
            Value::Uuid(value) => value.to_string(),
            Value::Json(value) => value.to_string(),
            Value::Array(value) => {
                let list_delimiter = t!("list_delimiter", locale = locale.name()).to_string();
                value
                    .iter()
                    .map(|value| value.to_formatted_string(locale))
                    .collect::<Vec<String>>()
                    .join(list_delimiter.as_str())
            }
            Value::Map(value) => {
                let key_value_delimiter =
                    t!("key_value_delimiter", locale = locale.name()).to_string();
                let list_delimiter = t!("list_delimiter", locale = locale.name()).to_string();
                value
                    .iter()
                    .map(|(key, value)| {
                        format!(
                            "{}{}{}",
                            key.to_formatted_string(locale),
                            key_value_delimiter,
                            value.to_formatted_string(locale)
                        )
                    })
                    .collect::<Vec<String>>()
                    .join(list_delimiter.as_str())
            }
        }
    }

    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    #[must_use]
    pub fn is_numeric(&self) -> bool {
        #[expect(clippy::match_like_matches_macro)]
        match self {
            Value::I8(_) | Value::I16(_) | Value::I32(_) | Value::I64(_) | Value::I128(_) => true,
            Value::U8(_) | Value::U16(_) | Value::U32(_) | Value::U64(_) | Value::U128(_) => true,
            Value::F32(_) | Value::F64(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_value = match self {
            Value::Null => "null".to_string(),
            Value::Bool(value) => value.to_string(),
            Value::Bytes(bytes) => STANDARD.encode(bytes),
            Value::I8(value) => value.to_string(),
            Value::I16(value) => value.to_string(),
            Value::I32(value) => value.to_string(),
            Value::I64(value) => value.to_string(),
            Value::I128(value) => value.to_string(),
            Value::U8(value) => value.to_string(),
            Value::U16(value) => value.to_string(),
            Value::U32(value) => value.to_string(),
            Value::U64(value) => value.to_string(),
            Value::U128(value) => value.to_string(),
            Value::F32(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::String(value) => value.to_string(),
            Value::Date(value) => value.to_string(),
            Value::Time(value) => value.to_string(),
            Value::DateTime(value) => value.to_string(),
            Value::Uuid(value) => value.to_string(),
            Value::Json(value) => value.to_string(),
            Value::Array(value) => value
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(", "),
            Value::Map(value) => value
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<String>>()
                .join(", "),
        };
        write!(f, "{string_value}")
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => 0.hash(state),
            Value::Bool(value) => value.hash(state),
            Value::Bytes(value) => value.hash(state),
            Value::I8(value) => value.hash(state),
            Value::I16(value) => value.hash(state),
            Value::I32(value) => value.hash(state),
            Value::I64(value) => value.hash(state),
            Value::I128(value) => value.hash(state),
            Value::U8(value) => value.hash(state),
            Value::U16(value) => value.hash(state),
            Value::U32(value) => value.hash(state),
            Value::U64(value) => value.hash(state),
            Value::U128(value) => value.hash(state),
            Value::F32(value) => value.to_bits().hash(state),
            Value::F64(value) => value.to_bits().hash(state),
            Value::String(value) => value.hash(state),
            Value::Date(value) => value.hash(state),
            Value::Time(value) => value.hash(state),
            Value::DateTime(value) => value.hash(state),
            Value::Uuid(value) => value.hash(state),
            Value::Json(value) => value.hash(state),
            Value::Array(value) => value.hash(state),
            Value::Map(value) => {
                for (key, value) in value {
                    key.hash(state);
                    value.hash(state);
                }
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::I8(a), Value::I8(b)) => a == b,
            (Value::I16(a), Value::I16(b)) => a == b,
            (Value::I32(a), Value::I32(b)) => a == b,
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::I128(a), Value::I128(b)) => a == b,
            (Value::U8(a), Value::U8(b)) => a == b,
            (Value::U16(a), Value::U16(b)) => a == b,
            (Value::U32(a), Value::U32(b)) => a == b,
            (Value::U64(a), Value::U64(b)) => a == b,
            (Value::U128(a), Value::U128(b)) => a == b,
            (Value::F32(a), Value::F32(b)) => a == b,
            (Value::F64(a), Value::F64(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::Time(a), Value::Time(b)) => a == b,
            (Value::DateTime(a), Value::DateTime(b)) => a == b,
            (Value::Uuid(a), Value::Uuid(b)) => a == b,
            (Value::Json(a), Value::Json(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => {
                if a.len() != b.len() {
                    return false;
                }

                // Compare keys and values in an order-dependent manner
                a.iter()
                    .zip(b.iter())
                    .all(|((a_key, a_value), (b_key, b_value))| {
                        a_key == b_key && a_value == b_value
                    })
            }
            _ => false,
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_none(),
            Value::Bool(value) => serializer.serialize_bool(value),
            Value::Bytes(ref value) => serializer.serialize_bytes(value),
            Value::I8(value) => serializer.serialize_i8(value),
            Value::I16(value) => serializer.serialize_i16(value),
            Value::I32(value) => serializer.serialize_i32(value),
            Value::I64(value) => serializer.serialize_i64(value),
            Value::I128(value) => serializer.serialize_str(&value.to_string()),
            Value::U8(value) => serializer.serialize_u8(value),
            Value::U16(value) => serializer.serialize_u16(value),
            Value::U32(value) => serializer.serialize_u32(value),
            Value::U64(value) => serializer.serialize_u64(value),
            Value::U128(value) => serializer.serialize_str(&value.to_string()),
            Value::F32(value) => serializer.serialize_f32(value),
            Value::F64(value) => serializer.serialize_f64(value),
            Value::String(ref value) => serializer.serialize_str(value),
            Value::Date(value) => serializer.serialize_str(&value.to_string()),
            Value::Time(value) => serializer.serialize_str(&value.to_string()),
            Value::DateTime(value) => serializer.serialize_str(&value.to_string()),
            Value::Uuid(value) => serializer.serialize_str(&value.to_string()),
            Value::Json(ref value) => value.serialize(serializer),
            Value::Array(ref value) => value.serialize(serializer),
            Value::Map(ref value) => value.serialize(serializer),
        }
    }
}

impl From<Option<Value>> for Value {
    fn from(value: Option<Value>) -> Self {
        value.unwrap_or(Value::Null)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Bytes(value)
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::I8(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::U8(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::I16(value)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::U16(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::I32(value)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::U32(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::U64(value)
    }
}

impl From<isize> for Value {
    fn from(value: isize) -> Self {
        Value::I64(value as i64)
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::U64(value as u64)
    }
}

impl From<i128> for Value {
    fn from(value: i128) -> Self {
        Value::I128(value)
    }
}

impl From<u128> for Value {
    fn from(value: u128) -> Self {
        Value::U128(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::F32(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::F64(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<chrono::NaiveDate> for Value {
    fn from(value: chrono::NaiveDate) -> Self {
        Value::Date(value)
    }
}

impl From<chrono::NaiveTime> for Value {
    fn from(value: chrono::NaiveTime) -> Self {
        Value::Time(value)
    }
}

impl From<chrono::NaiveDateTime> for Value {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Value::DateTime(value)
    }
}

impl From<uuid::Uuid> for Value {
    fn from(value: uuid::Uuid) -> Self {
        Value::Uuid(value)
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        Value::Json(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::Array(value)
    }
}

impl From<IndexMap<Value, Value>> for Value {
    fn from(value: IndexMap<Value, Value>) -> Self {
        Value::Map(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use serde_json::json;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn test_null() {
        assert!(Value::Null.is_null());
        assert!(!Value::Null.is_numeric());
        assert_eq!(Value::Null.to_formatted_string(&Locale::en), "null");
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(json!(Value::Null), json!(serde_json::Value::Null));
    }

    #[test]
    fn test_bool() {
        assert!(!Value::Bool(true).is_null());
        assert!(!Value::Bool(true).is_numeric());
        assert_eq!(Value::Bool(true).to_formatted_string(&Locale::en), "true");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(json!(Value::Bool(true)), json!(true));
    }

    #[test]
    fn test_bytes() {
        assert!(!Value::Bytes(vec![114, 117, 115, 116]).is_null());
        assert!(!Value::Bytes(vec![114, 117, 115, 116]).is_numeric());
        assert_eq!(
            Value::Bytes(vec![114, 117, 115, 116]).to_formatted_string(&Locale::en),
            "cnVzdA=="
        );
        assert_eq!(
            Value::Bytes(vec![114, 117, 115, 116]).to_string(),
            "cnVzdA=="
        );
        assert_eq!(
            json!(Value::Bytes(vec![114, 117, 115, 116])),
            serde_json::Value::Array(vec![
                serde_json::Value::Number(serde_json::Number::from(114)),
                serde_json::Value::Number(serde_json::Number::from(117)),
                serde_json::Value::Number(serde_json::Number::from(115)),
                serde_json::Value::Number(serde_json::Number::from(116))
            ])
        );
    }

    #[test]
    fn test_i8() {
        assert!(!Value::I8(i8::MIN).is_null());
        assert!(Value::I8(i8::MIN).is_numeric());
        assert_eq!(Value::I8(i8::MIN).to_formatted_string(&Locale::en), "-128");
        assert_eq!(Value::I8(i8::MAX).to_formatted_string(&Locale::en), "127");

        assert_eq!(Value::I8(i8::MIN).to_string(), "-128");
        assert_eq!(Value::I8(i8::MAX).to_string(), "127");

        assert_eq!(json!(Value::I8(i8::MIN)), json!(i8::MIN));
        assert_eq!(json!(Value::I8(i8::MAX)), json!(i8::MAX));
    }

    #[test]
    fn test_i16() {
        assert!(!Value::I16(i16::MIN).is_null());
        assert!(Value::I16(i16::MIN).is_numeric());
        assert_eq!(
            Value::I16(i16::MIN).to_formatted_string(&Locale::en),
            "-32,768"
        );
        assert_eq!(
            Value::I16(i16::MAX).to_formatted_string(&Locale::en),
            "32,767"
        );

        assert_eq!(Value::I16(i16::MIN).to_string(), "-32768");
        assert_eq!(Value::I16(i16::MAX).to_string(), "32767");

        assert_eq!(json!(Value::I16(i16::MIN)), json!(i16::MIN));
        assert_eq!(json!(Value::I16(i16::MAX)), json!(i16::MAX));
    }

    #[test]
    fn test_i32() {
        assert!(!Value::I32(i32::MIN).is_null());
        assert!(Value::I32(i32::MIN).is_numeric());
        assert_eq!(
            Value::I32(i32::MIN).to_formatted_string(&Locale::en),
            "-2,147,483,648"
        );
        assert_eq!(
            Value::I32(i32::MAX).to_formatted_string(&Locale::en),
            "2,147,483,647"
        );

        assert_eq!(Value::I32(i32::MIN).to_string(), "-2147483648");
        assert_eq!(Value::I32(i32::MAX).to_string(), "2147483647");

        assert_eq!(json!(Value::I32(i32::MIN)), json!(i32::MIN));
        assert_eq!(json!(Value::I32(i32::MAX)), json!(i32::MAX));
    }

    #[test]
    fn test_i64() {
        assert!(!Value::I64(i64::MIN).is_null());
        assert!(Value::I64(i64::MIN).is_numeric());
        assert_eq!(
            Value::I64(i64::MIN).to_formatted_string(&Locale::en),
            "-9,223,372,036,854,775,808"
        );
        assert_eq!(
            Value::I64(i64::MAX).to_formatted_string(&Locale::en),
            "9,223,372,036,854,775,807"
        );

        assert_eq!(Value::I64(i64::MIN).to_string(), "-9223372036854775808");
        assert_eq!(Value::I64(i64::MAX).to_string(), "9223372036854775807");

        assert_eq!(json!(Value::I64(i64::MIN)), json!(i64::MIN));
        assert_eq!(json!(Value::I64(i64::MAX)), json!(i64::MAX));
    }

    #[test]
    fn test_i128() {
        assert!(!Value::I128(i128::MIN).is_null());
        assert!(Value::I128(i128::MIN).is_numeric());
        assert_eq!(
            Value::I128(i128::MIN).to_formatted_string(&Locale::en),
            "-170,141,183,460,469,231,731,687,303,715,884,105,728"
        );
        assert_eq!(
            Value::I128(i128::MAX).to_formatted_string(&Locale::en),
            "170,141,183,460,469,231,731,687,303,715,884,105,727"
        );

        assert_eq!(
            Value::I128(i128::MIN).to_string(),
            "-170141183460469231731687303715884105728"
        );
        assert_eq!(
            Value::I128(i128::MAX).to_string(),
            "170141183460469231731687303715884105727"
        );
    }

    #[test]
    fn test_u8() {
        assert!(!Value::U8(u8::MAX).is_null());
        assert!(Value::U8(u8::MAX).is_numeric());
        assert_eq!(Value::U8(u8::MAX).to_formatted_string(&Locale::en), "255");
        assert_eq!(Value::U8(u8::MAX).to_string(), "255");
        assert_eq!(json!(Value::U8(u8::MAX)), json!(u8::MAX));
    }

    #[test]
    fn test_u16() {
        assert!(!Value::U16(u16::MAX).is_null());
        assert!(Value::U16(u16::MAX).is_numeric());
        assert_eq!(
            Value::U16(u16::MAX).to_formatted_string(&Locale::en),
            "65,535"
        );
        assert_eq!(Value::U16(u16::MAX).to_string(), "65535");
        assert_eq!(json!(Value::U16(u16::MAX)), json!(u16::MAX));
    }

    #[test]
    fn test_u32() {
        assert!(!Value::U32(u32::MAX).is_null());
        assert!(Value::U32(u32::MAX).is_numeric());
        assert_eq!(
            Value::U32(u32::MAX).to_formatted_string(&Locale::en),
            "4,294,967,295"
        );
        assert_eq!(Value::U32(u32::MAX).to_string(), "4294967295");
        assert_eq!(json!(Value::U32(u32::MAX)), json!(u32::MAX));
    }

    #[test]
    fn test_u64() {
        assert!(!Value::U64(u64::MAX).is_null());
        assert!(Value::U64(u64::MAX).is_numeric());
        assert_eq!(
            Value::U64(u64::MAX).to_formatted_string(&Locale::en),
            "18,446,744,073,709,551,615"
        );
        assert_eq!(Value::U64(u64::MAX).to_string(), "18446744073709551615");
        assert_eq!(json!(Value::U64(u64::MAX)), json!(u64::MAX));
    }

    #[test]
    fn test_u128() {
        assert!(!Value::U128(u128::MAX).is_null());
        assert!(Value::U128(u128::MAX).is_numeric());
        assert_eq!(
            Value::U128(u128::MAX).to_formatted_string(&Locale::en),
            "340,282,366,920,938,463,463,374,607,431,768,211,455"
        );
        assert_eq!(
            Value::U128(u128::MAX).to_string(),
            "340282366920938463463374607431768211455"
        );
    }

    #[test]
    fn test_f32() {
        assert!(!Value::F32(12_345.678).is_null());
        assert!(Value::F32(12_345.678).is_numeric());
        assert!(Value::F32(12_345.678)
            .to_formatted_string(&Locale::en)
            .starts_with("12345."));
        assert!(Value::F32(12_345.678).to_string().starts_with("12345."));
        assert_eq!(json!(Value::F32(12_345.0)), json!(12_345.0));
    }

    #[test]
    fn test_f64() {
        assert!(!Value::F64(12_345.678_90).is_null());
        assert!(Value::F64(12_345.678_90).is_numeric());
        assert!(Value::F64(12_345.678_90)
            .to_formatted_string(&Locale::en)
            .starts_with("12345."));
        assert!(Value::F64(12_345.678_90).to_string().starts_with("12345."));
        assert_eq!(json!(Value::F64(12_345.678_90)), json!(12_345.678_90));
    }

    #[test]
    fn test_string() {
        assert!(!Value::String("foo".to_string()).is_null());
        assert!(!Value::String("foo".to_string()).is_numeric());
        assert_eq!(
            Value::String("foo".to_string()).to_formatted_string(&Locale::en),
            "foo"
        );
        assert_eq!(Value::String("foo".to_string()).to_string(), "foo");
        assert_eq!(
            json!(Value::String("foo".to_string())),
            json!("foo".to_string())
        );
    }

    #[test]
    fn test_date() {
        let date = NaiveDate::from_ymd_opt(2000, 12, 31).expect("Invalid date");
        assert!(!Value::Date(date).is_null());
        assert!(!Value::Date(date).is_numeric());
        assert_eq!(
            Value::Date(date).to_formatted_string(&Locale::en),
            "2000-12-31"
        );
        assert_eq!(Value::Date(date).to_string(), "2000-12-31");
        assert_eq!(json!(Value::Date(date)), json!("2000-12-31"));
    }

    #[test]
    fn test_time() {
        let time = NaiveTime::from_hms_milli_opt(12, 13, 14, 15).expect("Invalid time");
        assert!(!Value::Time(time).is_null());
        assert!(!Value::Time(time).is_numeric());
        assert_eq!(
            Value::Time(time).to_formatted_string(&Locale::en),
            "12:13:14.015"
        );
        assert_eq!(Value::Time(time).to_string(), "12:13:14.015");
        assert_eq!(json!(Value::Time(time)), json!("12:13:14.015"));
    }

    #[test]
    fn test_datetime() {
        let date = NaiveDate::from_ymd_opt(2000, 12, 31).expect("Invalid date");
        let time = NaiveTime::from_hms_milli_opt(12, 13, 14, 15).expect("Invalid time");
        let datetime = NaiveDateTime::new(date, time);
        assert!(!Value::DateTime(datetime).is_null());
        assert!(!Value::DateTime(datetime).is_numeric());
        assert_eq!(
            Value::DateTime(datetime).to_formatted_string(&Locale::en),
            "2000-12-31 12:13:14.015"
        );
        assert_eq!(
            Value::DateTime(datetime).to_string(),
            "2000-12-31 12:13:14.015"
        );
        assert_eq!(
            json!(Value::DateTime(datetime)),
            json!("2000-12-31 12:13:14.015")
        );
    }

    #[test]
    fn test_uuid() -> Result<()> {
        let uuid = "acf5b3e3-4099-4f34-81c7-5803cbc87a2d";
        assert!(!Value::Uuid(Uuid::from_str(uuid)?).is_null());
        assert!(!Value::Uuid(Uuid::from_str(uuid)?).is_numeric());
        assert_eq!(
            Value::Uuid(Uuid::from_str(uuid)?).to_formatted_string(&Locale::en),
            uuid
        );
        assert_eq!(Value::Uuid(Uuid::from_str(uuid)?).to_string(), uuid);
        assert_eq!(json!(Value::Uuid(Uuid::from_str(uuid)?)), json!(uuid));
        Ok(())
    }

    #[test]
    fn test_json() {
        let original_json = json!({"foo": "bar", "baz": 123});
        assert!(!Value::Json(original_json.clone()).is_null());
        assert!(!Value::Json(original_json.clone()).is_numeric());
        assert_eq!(
            Value::Json(original_json.clone()).to_formatted_string(&Locale::en),
            r#"{"foo":"bar","baz":123}"#
        );
        assert_eq!(
            Value::Json(original_json.clone()).to_string(),
            r#"{"foo":"bar","baz":123}"#
        );
        assert_eq!(
            json!(Value::Json(original_json.clone())),
            json!({"foo":"bar","baz":123})
        );
    }

    #[test]
    fn test_array() -> Result<()> {
        let array = vec![
            Value::Null,
            Value::Bool(true),
            Value::I8(1),
            Value::I16(2),
            Value::I32(3),
            Value::I64(12345),
            Value::I128(128),
            Value::U8(5),
            Value::U16(6),
            Value::U32(7),
            Value::U64(8),
            Value::U128(128),
            Value::F32(9.1),
            Value::F64(10.42),
            Value::String("foo".to_string()),
            Value::Date(NaiveDate::from_ymd_opt(2000, 12, 31).expect("Invalid date")),
            Value::Time(NaiveTime::from_hms_milli_opt(12, 13, 14, 15).expect("Invalid time")),
            Value::DateTime(NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2000, 12, 31).expect("Invalid date"),
                NaiveTime::from_hms_milli_opt(12, 13, 14, 15).expect("Invalid time"),
            )),
            Value::Uuid(Uuid::from_str("acf5b3e3-4099-4f34-81c7-5803cbc87a2d")?),
            Value::Json(json!({"foo": "bar", "baz": 123})),
        ];
        assert_eq!(
            Value::Array(array.clone()).to_formatted_string(&Locale::en),
            r#"null, true, 1, 2, 3, 12,345, 128, 5, 6, 7, 8, 128, 9.1, 10.42, foo, 2000-12-31, 12:13:14.015, 2000-12-31 12:13:14.015, acf5b3e3-4099-4f34-81c7-5803cbc87a2d, {"foo":"bar","baz":123}"#
        );
        assert_eq!(
            Value::Array(array.clone()).to_string(),
            r#"null, true, 1, 2, 3, 12345, 128, 5, 6, 7, 8, 128, 9.1, 10.42, foo, 2000-12-31, 12:13:14.015, 2000-12-31 12:13:14.015, acf5b3e3-4099-4f34-81c7-5803cbc87a2d, {"foo":"bar","baz":123}"#
        );
        assert_eq!(json!(Value::Array(array.clone())), json!(array.clone()));
        Ok(())
    }

    #[test]
    fn test_map() {
        let mut map = IndexMap::new();
        map.insert(Value::String("foo".to_string()), Value::I32(123));
        map.insert(Value::String("bar".to_string()), Value::I32(456));
        map.insert(Value::String("baz".to_string()), Value::I32(789));
        assert_eq!(
            "foo=123, bar=456, baz=789",
            Value::Map(map.clone()).to_formatted_string(&Locale::en)
        );
        assert_eq!(
            "foo=123, bar=456, baz=789",
            Value::Map(map.clone()).to_string()
        );
        assert_eq!(json!(Value::Map(map.clone())), json!(map.clone()));
    }

    #[test]
    fn test_from_option() {
        let value: Option<Value> = None;
        assert_eq!(Value::from(value), Value::Null);
    }

    #[test]
    fn test_from_bool() {
        assert_eq!(Value::from(true), Value::Bool(true));
    }

    #[test]
    fn test_from_vec_u8() {
        assert_eq!(Value::from(vec![42]), Value::Bytes(vec![42]));
    }

    #[test]
    fn test_from_i8() {
        assert_eq!(Value::from(i8::MIN), Value::I8(i8::MIN));
    }

    #[test]
    fn test_from_u8() {
        assert_eq!(Value::from(u8::MAX), Value::U8(u8::MAX));
    }

    #[test]
    fn test_from_i16() {
        assert_eq!(Value::from(i16::MIN), Value::I16(i16::MIN));
    }

    #[test]
    fn test_from_u16() {
        assert_eq!(Value::from(u16::MAX), Value::U16(u16::MAX));
    }

    #[test]
    fn test_from_i32() {
        assert_eq!(Value::from(i32::MIN), Value::I32(i32::MIN));
    }

    #[test]
    fn test_from_u32() {
        assert_eq!(Value::from(u32::MAX), Value::U32(u32::MAX));
    }

    #[test]
    fn test_from_i64() {
        assert_eq!(Value::from(i64::MIN), Value::I64(i64::MIN));
    }

    #[test]
    fn test_from_u64() {
        assert_eq!(Value::from(u64::MAX), Value::U64(u64::MAX));
    }

    #[test]
    fn test_from_isize() {
        assert_eq!(Value::from(isize::MIN), Value::I64(isize::MIN as i64));
    }

    #[test]
    fn test_from_usize() {
        assert_eq!(Value::from(usize::MAX), Value::U64(usize::MAX as u64));
    }

    #[test]
    fn test_from_i128() {
        assert_eq!(Value::from(i128::MIN), Value::I128(i128::MIN));
    }

    #[test]
    fn test_from_u128() {
        assert_eq!(Value::from(u128::MAX), Value::U128(u128::MAX));
    }

    #[test]
    fn test_from_f32() {
        assert_eq!(Value::from(42.1f32), Value::F32(42.1f32));
    }

    #[test]
    fn test_from_f64() {
        assert_eq!(Value::from(42.1f64), Value::F64(42.1f64));
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Value::from("foo"), Value::String("foo".to_string()));
    }

    #[test]
    fn test_from_string() {
        assert_eq!(
            Value::from("foo".to_string()),
            Value::String("foo".to_string())
        );
    }

    #[test]
    fn test_from_naive_date() {
        let date = NaiveDate::from_ymd_opt(2000, 12, 31).expect("Invalid date");
        assert_eq!(Value::from(date), Value::Date(date));
    }

    #[test]
    fn test_from_naive_time() {
        let time = NaiveTime::from_hms_milli_opt(12, 13, 14, 15).expect("Invalid time");
        assert_eq!(Value::from(time), Value::Time(time));
    }

    #[test]
    fn test_from_naive_date_time() {
        let date = NaiveDate::from_ymd_opt(2000, 12, 31).expect("Invalid date");
        let time = NaiveTime::from_hms_milli_opt(12, 13, 14, 15).expect("Invalid time");
        let datetime = NaiveDateTime::new(date, time);
        assert_eq!(Value::from(datetime), Value::DateTime(datetime));
    }

    #[test]
    fn test_from_uuid() -> Result<()> {
        let uuid = "acf5b3e3-4099-4f34-81c7-5803cbc87a2d";
        assert_eq!(
            Value::from(Uuid::from_str(uuid)?),
            Value::Uuid(Uuid::from_str(uuid)?)
        );
        Ok(())
    }

    #[test]
    fn test_from_json() {
        let json = json!({"foo": "bar", "baz": 123});
        assert_eq!(Value::from(json.clone()), Value::Json(json.clone()));
    }

    #[test]
    fn test_from_vec_value() {
        let array = vec![Value::Bool(true), Value::I8(42)];
        assert_eq!(Value::from(array.clone()), Value::Array(array.clone()));
    }

    #[test]
    fn test_from_index_map() {
        let mut map = IndexMap::new();
        map.insert(Value::String("foo".to_string()), Value::I32(123));
        assert_eq!(Value::from(map.clone()), Value::Map(map.clone()));
    }
}
