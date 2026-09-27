use rsql_driver::{Error, Result};

/// Type identifiers defined by `java.sql.Types`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub(crate) enum JdbcType {
    LongNVarchar = -16,
    NChar = -15,
    NVarchar = -9,
    RowId = -8,
    Bit = -7,
    TinyInt = -6,
    BigInt = -5,
    LongVarBinary = -4,
    VarBinary = -3,
    Binary = -2,
    LongVarchar = -1,
    Null = 0,
    Char = 1,
    Numeric = 2,
    Decimal = 3,
    Integer = 4,
    SmallInt = 5,
    Float = 6,
    Real = 7,
    Double = 8,
    Varchar = 12,
    Boolean = 16,
    DataLink = 70,
    Date = 91,
    Time = 92,
    Timestamp = 93,
    Other = 1111,
    JavaObject = 2000,
    Distinct = 2001,
    Struct = 2002,
    Array = 2003,
    Blob = 2004,
    Clob = 2005,
    Ref = 2006,
    SqlXml = 2009,
    NClob = 2011,
    RefCursor = 2012,
    TimeWithTimezone = 2013,
    TimestampWithTimezone = 2014,
}

impl TryFrom<i32> for JdbcType {
    type Error = Error;

    fn try_from(code: i32) -> Result<Self> {
        // Keep wire codes centralized here; conversions use enum variants exclusively.
        let types = [
            Self::LongNVarchar,
            Self::NChar,
            Self::NVarchar,
            Self::RowId,
            Self::Bit,
            Self::TinyInt,
            Self::BigInt,
            Self::LongVarBinary,
            Self::VarBinary,
            Self::Binary,
            Self::LongVarchar,
            Self::Null,
            Self::Char,
            Self::Numeric,
            Self::Decimal,
            Self::Integer,
            Self::SmallInt,
            Self::Float,
            Self::Real,
            Self::Double,
            Self::Varchar,
            Self::Boolean,
            Self::DataLink,
            Self::Date,
            Self::Time,
            Self::Timestamp,
            Self::Other,
            Self::JavaObject,
            Self::Distinct,
            Self::Struct,
            Self::Array,
            Self::Blob,
            Self::Clob,
            Self::Ref,
            Self::SqlXml,
            Self::NClob,
            Self::RefCursor,
            Self::TimeWithTimezone,
            Self::TimestampWithTimezone,
        ];
        types
            .into_iter()
            .find(|jdbc_type| *jdbc_type as i32 == code)
            .ok_or_else(|| Error::ConversionError(format!("Unknown JDBC type code: {code}")))
    }
}
