use num_format::{Locale, ToFormattedString};

pub(crate) enum Value {
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
            Value::Bytes(bytes) => format!("{:?}", bytes),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes() {
        assert_eq!(
            Value::Bytes(vec![42]).to_formatted_string(&Locale::en),
            "[42]"
        );
    }

    #[test]
    fn test_i8() {
        assert_eq!(Value::I8(-128).to_formatted_string(&Locale::en), "-128");
        assert_eq!(Value::I8(127).to_formatted_string(&Locale::en), "127");
    }

    #[test]
    fn test_i16() {
        assert_eq!(
            Value::I16(-32_768).to_formatted_string(&Locale::en),
            "-32,768"
        );
        assert_eq!(
            Value::I16(32_767).to_formatted_string(&Locale::en),
            "32,767"
        );
    }

    #[test]
    fn test_i32() {
        assert_eq!(
            Value::I32(-2_147_483_648).to_formatted_string(&Locale::en),
            "-2,147,483,648"
        );
        assert_eq!(
            Value::I32(2_147_483_647).to_formatted_string(&Locale::en),
            "2,147,483,647"
        );
    }

    #[test]
    fn test_i64() {
        assert_eq!(
            Value::I64(-9_223_372_036_854_775_808).to_formatted_string(&Locale::en),
            "-9,223,372,036,854,775,808"
        );
        assert_eq!(
            Value::I64(9_223_372_036_854_775_807).to_formatted_string(&Locale::en),
            "9,223,372,036,854,775,807"
        );
    }

    #[test]
    fn test_u8() {
        assert_eq!(Value::U8(255).to_formatted_string(&Locale::en), "255");
    }

    #[test]
    fn test_u16() {
        assert_eq!(
            Value::U16(65_535).to_formatted_string(&Locale::en),
            "65,535"
        );
    }

    #[test]
    fn test_u32() {
        assert_eq!(
            Value::U32(4_294_967_295).to_formatted_string(&Locale::en),
            "4,294,967,295"
        );
    }

    #[test]
    fn test_u64() {
        assert_eq!(
            Value::U64(18_446_744_073_709_551_615).to_formatted_string(&Locale::en),
            "18,446,744,073,709,551,615"
        );
    }

    #[test]
    fn test_f32() {
        assert!(Value::F32(12_345.67890)
            .to_formatted_string(&Locale::en)
            .starts_with("12345."));
    }

    #[test]
    fn test_f64() {
        assert!(Value::F64(12_345.67890)
            .to_formatted_string(&Locale::en)
            .starts_with("12345."));
    }

    #[test]
    fn test_string() {
        assert_eq!(
            Value::String("foo".to_string()).to_formatted_string(&Locale::en),
            "foo"
        );
    }
}
