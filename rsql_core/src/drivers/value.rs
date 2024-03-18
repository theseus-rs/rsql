use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use num_format::{Locale, ToFormattedString};
use serde::{Serialize, Serializer};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Bytes(Vec<u8>),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    #[allow(dead_code)]
    U8(u8),
    #[allow(dead_code)]
    U16(u16),
    #[allow(dead_code)]
    U32(u32),
    #[allow(dead_code)]
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Date(chrono::NaiveDate),
    Time(chrono::NaiveTime),
    DateTime(chrono::NaiveDateTime),
    Uuid(uuid::Uuid),
    Json(serde_json::Value),
}

impl Value {
    pub(crate) fn to_formatted_string(&self, locale: &Locale) -> String {
        match self {
            Value::Bool(value) => value.to_string(),
            Value::Bytes(bytes) => STANDARD.encode(bytes),
            Value::I8(value) => value.to_formatted_string(locale),
            Value::I16(value) => value.to_formatted_string(locale),
            Value::I32(value) => value.to_formatted_string(locale),
            Value::I64(value) => value.to_formatted_string(locale),
            Value::U8(value) => value.to_formatted_string(locale),
            Value::U16(value) => value.to_formatted_string(locale),
            Value::U32(value) => value.to_formatted_string(locale),
            Value::U64(value) => value.to_formatted_string(locale),
            Value::F32(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::String(value) => value.to_string(),
            Value::Date(value) => value.to_string(),
            Value::Time(value) => value.to_string(),
            Value::DateTime(value) => value.to_string(),
            Value::Uuid(value) => value.to_string(),
            Value::Json(value) => value.to_string(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string_value = match self {
            Value::Bool(value) => value.to_string(),
            Value::Bytes(bytes) => STANDARD.encode(bytes),
            Value::I8(value) => value.to_string(),
            Value::I16(value) => value.to_string(),
            Value::I32(value) => value.to_string(),
            Value::I64(value) => value.to_string(),
            Value::U8(value) => value.to_string(),
            Value::U16(value) => value.to_string(),
            Value::U32(value) => value.to_string(),
            Value::U64(value) => value.to_string(),
            Value::F32(value) => value.to_string(),
            Value::F64(value) => value.to_string(),
            Value::String(value) => value.to_string(),
            Value::Date(value) => value.to_string(),
            Value::Time(value) => value.to_string(),
            Value::DateTime(value) => value.to_string(),
            Value::Uuid(value) => value.to_string(),
            Value::Json(value) => value.to_string(),
        };
        write!(f, "{}", string_value)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Value::Bool(value) => serializer.serialize_bool(value),
            Value::Bytes(ref value) => serializer.serialize_bytes(value),
            Value::I8(value) => serializer.serialize_i8(value),
            Value::I16(value) => serializer.serialize_i16(value),
            Value::I32(value) => serializer.serialize_i32(value),
            Value::I64(value) => serializer.serialize_i64(value),
            Value::U8(value) => serializer.serialize_u8(value),
            Value::U16(value) => serializer.serialize_u16(value),
            Value::U32(value) => serializer.serialize_u32(value),
            Value::U64(value) => serializer.serialize_u64(value),
            Value::F32(value) => serializer.serialize_f32(value),
            Value::F64(value) => serializer.serialize_f64(value),
            Value::String(ref value) => serializer.serialize_str(value),
            Value::Date(value) => serializer.serialize_str(&value.to_string()),
            Value::Time(value) => serializer.serialize_str(&value.to_string()),
            Value::DateTime(value) => serializer.serialize_str(&value.to_string()),
            Value::Uuid(value) => serializer.serialize_str(&value.to_string()),
            Value::Json(ref value) => value.serialize(serializer),
        }
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
    fn test_bool() {
        assert_eq!(Value::Bool(true).to_formatted_string(&Locale::en), "true");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(json!(Value::Bool(true)), json!(true));
    }

    #[test]
    fn test_bytes() {
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
        assert_eq!(Value::I8(i8::MIN).to_formatted_string(&Locale::en), "-128");
        assert_eq!(Value::I8(i8::MAX).to_formatted_string(&Locale::en), "127");

        assert_eq!(Value::I8(i8::MIN).to_string(), "-128");
        assert_eq!(Value::I8(i8::MAX).to_string(), "127");

        assert_eq!(json!(Value::I8(i8::MIN)), json!(i8::MIN));
        assert_eq!(json!(Value::I8(i8::MAX)), json!(i8::MAX));
    }

    #[test]
    fn test_i16() {
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
    fn test_u8() {
        assert_eq!(Value::U8(u8::MAX).to_formatted_string(&Locale::en), "255");
        assert_eq!(Value::U8(u8::MAX).to_string(), "255");
        assert_eq!(json!(Value::U8(u8::MAX)), json!(u8::MAX));
    }

    #[test]
    fn test_u16() {
        assert_eq!(
            Value::U16(u16::MAX).to_formatted_string(&Locale::en),
            "65,535"
        );
        assert_eq!(Value::U16(u16::MAX).to_string(), "65535");
        assert_eq!(json!(Value::U16(u16::MAX)), json!(u16::MAX));
    }

    #[test]
    fn test_u32() {
        assert_eq!(
            Value::U32(u32::MAX).to_formatted_string(&Locale::en),
            "4,294,967,295"
        );
        assert_eq!(Value::U32(u32::MAX).to_string(), "4294967295");
        assert_eq!(json!(Value::U32(u32::MAX)), json!(u32::MAX));
    }

    #[test]
    fn test_u64() {
        assert_eq!(
            Value::U64(u64::MAX).to_formatted_string(&Locale::en),
            "18,446,744,073,709,551,615"
        );
        assert_eq!(Value::U64(u64::MAX).to_string(), "18446744073709551615");
        assert_eq!(json!(Value::U64(u64::MAX)), json!(u64::MAX));
    }

    #[test]
    fn test_f32() {
        assert!(Value::F32(12_345.67890)
            .to_formatted_string(&Locale::en)
            .starts_with("12345."));
        assert!(Value::F32(12_345.67890).to_string().starts_with("12345."));
        assert_eq!(json!(Value::F32(12_345.0)), json!(12_345.0));
    }

    #[test]
    fn test_f64() {
        assert!(Value::F64(12_345.67890)
            .to_formatted_string(&Locale::en)
            .starts_with("12345."));
        assert!(Value::F64(12_345.67890).to_string().starts_with("12345."));
        assert_eq!(json!(Value::F64(12_345.67890)), json!(12_345.67890));
    }

    #[test]
    fn test_string() {
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
        let date = NaiveDate::from_ymd_opt(2000, 12, 31).unwrap();
        assert_eq!(
            Value::Date(date).to_formatted_string(&Locale::en),
            "2000-12-31"
        );
        assert_eq!(Value::Date(date).to_string(), "2000-12-31");
        assert_eq!(json!(Value::Date(date)), json!("2000-12-31"));
    }

    #[test]
    fn test_time() {
        let time = NaiveTime::from_hms_milli_opt(12, 13, 14, 15).unwrap();
        assert_eq!(
            Value::Time(time).to_formatted_string(&Locale::en),
            "12:13:14.015"
        );
        assert_eq!(Value::Time(time).to_string(), "12:13:14.015");
        assert_eq!(json!(Value::Time(time)), json!("12:13:14.015"));
    }

    #[test]
    fn test_datetime() {
        let date = NaiveDate::from_ymd_opt(2000, 12, 31).unwrap();
        let time = NaiveTime::from_hms_milli_opt(12, 13, 14, 15).unwrap();
        let datetime = NaiveDateTime::new(date, time);
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
        assert_eq!(
            Value::Uuid(Uuid::from_str(uuid)?).to_formatted_string(&Locale::en),
            uuid
        );
        assert_eq!(Value::Uuid(Uuid::from_str(uuid)?).to_string(), uuid);
        assert_eq!(json!(Value::Uuid(Uuid::from_str(uuid)?)), json!(uuid));
        Ok(())
    }

    #[test]
    fn test_json() -> Result<()> {
        let original_json = json!({"foo": "bar", "baz": 123});
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
        Ok(())
    }
}
