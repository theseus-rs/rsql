use crate::Value;

/// Trait for types that can be converted to a SQL parameter value.
pub trait ToSql: Send + Sync {
    /// Convert this value to a [`Value`] for use as a SQL parameter.
    fn to_value(&self) -> Value;
}

/// Convert a slice of [`ToSql`] references into a [`Vec<Value>`] for use with
/// [`Connection::execute`](crate::Connection::execute) and
/// [`Connection::query`](crate::Connection::query).
pub fn to_values(params: &[&dyn ToSql]) -> Vec<Value> {
    params.iter().map(|p| p.to_value()).collect()
}

impl ToSql for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl ToSql for bool {
    fn to_value(&self) -> Value {
        Value::Bool(*self)
    }
}

impl ToSql for i8 {
    fn to_value(&self) -> Value {
        Value::I8(*self)
    }
}

impl ToSql for i16 {
    fn to_value(&self) -> Value {
        Value::I16(*self)
    }
}

impl ToSql for i32 {
    fn to_value(&self) -> Value {
        Value::I32(*self)
    }
}

impl ToSql for i64 {
    fn to_value(&self) -> Value {
        Value::I64(*self)
    }
}

impl ToSql for i128 {
    fn to_value(&self) -> Value {
        Value::I128(*self)
    }
}

impl ToSql for u8 {
    fn to_value(&self) -> Value {
        Value::U8(*self)
    }
}

impl ToSql for u16 {
    fn to_value(&self) -> Value {
        Value::U16(*self)
    }
}

impl ToSql for u32 {
    fn to_value(&self) -> Value {
        Value::U32(*self)
    }
}

impl ToSql for u64 {
    fn to_value(&self) -> Value {
        Value::U64(*self)
    }
}

impl ToSql for u128 {
    fn to_value(&self) -> Value {
        Value::U128(*self)
    }
}

impl ToSql for f32 {
    fn to_value(&self) -> Value {
        Value::F32(*self)
    }
}

impl ToSql for f64 {
    fn to_value(&self) -> Value {
        Value::F64(*self)
    }
}

impl ToSql for str {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToSql for String {
    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl ToSql for [u8] {
    fn to_value(&self) -> Value {
        Value::Bytes(self.to_vec())
    }
}

impl ToSql for Vec<u8> {
    fn to_value(&self) -> Value {
        Value::Bytes(self.clone())
    }
}

impl<T: ToSql> ToSql for Option<T> {
    fn to_value(&self) -> Value {
        match self {
            Some(v) => v.to_value(),
            None => Value::Null,
        }
    }
}

impl<T: ToSql + ?Sized> ToSql for &T {
    fn to_value(&self) -> Value {
        (**self).to_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_sql_value() {
        let value = Value::I32(42);
        assert_eq!(value.to_value(), Value::I32(42));
    }

    #[test]
    fn test_to_sql_value_null() {
        let value = Value::Null;
        assert_eq!(value.to_value(), Value::Null);
    }

    #[test]
    fn test_to_sql_bool() {
        assert_eq!(true.to_value(), Value::Bool(true));
        assert_eq!(false.to_value(), Value::Bool(false));
    }

    #[test]
    fn test_to_sql_i8() {
        assert_eq!(42i8.to_value(), Value::I8(42));
        assert_eq!(i8::MIN.to_value(), Value::I8(i8::MIN));
        assert_eq!(i8::MAX.to_value(), Value::I8(i8::MAX));
    }

    #[test]
    fn test_to_sql_i16() {
        assert_eq!(42i16.to_value(), Value::I16(42));
        assert_eq!(i16::MIN.to_value(), Value::I16(i16::MIN));
        assert_eq!(i16::MAX.to_value(), Value::I16(i16::MAX));
    }

    #[test]
    fn test_to_sql_i32() {
        assert_eq!(42i32.to_value(), Value::I32(42));
        assert_eq!(i32::MIN.to_value(), Value::I32(i32::MIN));
        assert_eq!(i32::MAX.to_value(), Value::I32(i32::MAX));
    }

    #[test]
    fn test_to_sql_i64() {
        assert_eq!(42i64.to_value(), Value::I64(42));
        assert_eq!(i64::MIN.to_value(), Value::I64(i64::MIN));
        assert_eq!(i64::MAX.to_value(), Value::I64(i64::MAX));
    }

    #[test]
    fn test_to_sql_i128() {
        assert_eq!(42i128.to_value(), Value::I128(42));
        assert_eq!(i128::MIN.to_value(), Value::I128(i128::MIN));
        assert_eq!(i128::MAX.to_value(), Value::I128(i128::MAX));
    }

    #[test]
    fn test_to_sql_u8() {
        assert_eq!(42u8.to_value(), Value::U8(42));
        assert_eq!(u8::MIN.to_value(), Value::U8(u8::MIN));
        assert_eq!(u8::MAX.to_value(), Value::U8(u8::MAX));
    }

    #[test]
    fn test_to_sql_u16() {
        assert_eq!(42u16.to_value(), Value::U16(42));
        assert_eq!(u16::MIN.to_value(), Value::U16(u16::MIN));
        assert_eq!(u16::MAX.to_value(), Value::U16(u16::MAX));
    }

    #[test]
    fn test_to_sql_u32() {
        assert_eq!(42u32.to_value(), Value::U32(42));
        assert_eq!(u32::MIN.to_value(), Value::U32(u32::MIN));
        assert_eq!(u32::MAX.to_value(), Value::U32(u32::MAX));
    }

    #[test]
    fn test_to_sql_u64() {
        assert_eq!(42u64.to_value(), Value::U64(42));
        assert_eq!(u64::MIN.to_value(), Value::U64(u64::MIN));
        assert_eq!(u64::MAX.to_value(), Value::U64(u64::MAX));
    }

    #[test]
    fn test_to_sql_u128() {
        assert_eq!(42u128.to_value(), Value::U128(42));
        assert_eq!(u128::MIN.to_value(), Value::U128(u128::MIN));
        assert_eq!(u128::MAX.to_value(), Value::U128(u128::MAX));
    }

    #[test]
    fn test_to_sql_f32() {
        assert_eq!(1.5f32.to_value(), Value::F32(1.5));
        assert_eq!(0.0f32.to_value(), Value::F32(0.0));
        assert_eq!(f32::MIN.to_value(), Value::F32(f32::MIN));
        assert_eq!(f32::MAX.to_value(), Value::F32(f32::MAX));
    }

    #[test]
    fn test_to_sql_f64() {
        assert_eq!(1.5f64.to_value(), Value::F64(1.5));
        assert_eq!(0.0f64.to_value(), Value::F64(0.0));
        assert_eq!(f64::MIN.to_value(), Value::F64(f64::MIN));
        assert_eq!(f64::MAX.to_value(), Value::F64(f64::MAX));
    }

    #[test]
    fn test_to_sql_str() {
        assert_eq!("hello".to_value(), Value::String("hello".to_string()));
        assert_eq!("".to_value(), Value::String(String::new()));
    }

    #[test]
    fn test_to_sql_string() {
        assert_eq!(
            String::from("hello").to_value(),
            Value::String("hello".to_string())
        );
        assert_eq!(String::new().to_value(), Value::String(String::new()));
    }

    #[test]
    fn test_to_sql_bytes_slice() {
        let bytes: &[u8] = &[1, 2, 3];
        assert_eq!(bytes.to_value(), Value::Bytes(vec![1, 2, 3]));
        let empty: &[u8] = &[];
        assert_eq!(empty.to_value(), Value::Bytes(vec![]));
    }

    #[test]
    fn test_to_sql_vec_u8() {
        assert_eq!(vec![1u8, 2, 3].to_value(), Value::Bytes(vec![1, 2, 3]));
        assert_eq!(Vec::<u8>::new().to_value(), Value::Bytes(vec![]));
    }

    #[test]
    fn test_to_sql_option_some() {
        assert_eq!(Some(42i32).to_value(), Value::I32(42));
        assert_eq!(
            Some("hello".to_string()).to_value(),
            Value::String("hello".to_string())
        );
        assert_eq!(Some(true).to_value(), Value::Bool(true));
    }

    #[test]
    fn test_to_sql_option_none() {
        assert_eq!(Option::<i32>::None.to_value(), Value::Null);
        assert_eq!(Option::<String>::None.to_value(), Value::Null);
        assert_eq!(Option::<bool>::None.to_value(), Value::Null);
    }

    #[test]
    fn test_to_sql_reference() {
        let val = 42i32;
        assert_eq!(val.to_value(), Value::I32(42));
        let s = String::from("hello");
        assert_eq!(s.to_value(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_to_sql_dyn_trait() {
        let params: Vec<&dyn ToSql> = vec![&42i32, &"hello", &true];
        assert_eq!(params[0].to_value(), Value::I32(42));
        assert_eq!(params[1].to_value(), Value::String("hello".to_string()));
        assert_eq!(params[2].to_value(), Value::Bool(true));
    }

    #[test]
    fn test_to_sql_dyn_trait_empty() {
        let params: Vec<&dyn ToSql> = vec![];
        assert!(params.is_empty());
    }
}
