use crate::connection::{invoke, jdbc_error};
use crate::jdbc_type::JdbcType;
use ristretto_vm::{VM, Value as JavaValue};
use rsql_driver::{MemoryQueryResult, QueryResult, Result, Value};

pub(crate) async fn read_result(vm: &VM, result_set: &JavaValue) -> Result<MemoryQueryResult> {
    let metadata = invoke(
        vm,
        result_set,
        "getMetaData()Ljava/sql/ResultSetMetaData;",
        &[],
    )
    .await?;
    let count = invoke(vm, &metadata, "getColumnCount()I", &[])
        .await?
        .as_i32()
        .map_err(jdbc_error)?;
    let mut columns = Vec::new();
    let mut types = Vec::new();
    for index in 1..=count {
        let argument = [JavaValue::Int(index)];
        columns.push(
            invoke(
                vm,
                &metadata,
                "getColumnLabel(I)Ljava/lang/String;",
                &argument,
            )
            .await?
            .as_string()
            .map_err(jdbc_error)?,
        );
        let jdbc_type = JdbcType::try_from(
            invoke(vm, &metadata, "getColumnType(I)I", &argument)
                .await?
                .as_i32()
                .map_err(jdbc_error)?,
        )?;
        let name = invoke(
            vm,
            &metadata,
            "getColumnTypeName(I)Ljava/lang/String;",
            &argument,
        )
        .await?
        .as_string()
        .map_err(jdbc_error)?;
        types.push((jdbc_type, name));
    }
    let mut rows = Vec::new();
    while invoke(vm, result_set, "next()Z", &[])
        .await?
        .as_bool()
        .map_err(jdbc_error)?
    {
        let mut row = Vec::with_capacity(columns.len());
        for (index, (column_type, name)) in types.iter().enumerate() {
            row.push(
                read_value(
                    vm,
                    result_set,
                    i32::try_from(index + 1)?,
                    *column_type,
                    name,
                )
                .await?,
            );
        }
        rows.push(row);
    }
    Ok(MemoryQueryResult::new(columns, rows))
}

async fn read_value(
    vm: &VM,
    result: &JavaValue,
    index: i32,
    column_type: JdbcType,
    name: &str,
) -> Result<Value> {
    let arguments = [JavaValue::Int(index)];
    if column_type == JdbcType::Array {
        return read_array(vm, result, &arguments).await;
    }
    if matches!(
        name.to_ascii_lowercase().as_str(),
        "uuid" | "json" | "jsonb" | "bit" | "varbit" | "timetz" | "timestamptz"
    ) {
        let value = invoke(vm, result, "getString(I)Ljava/lang/String;", &arguments).await?;
        if value.is_null() {
            return Ok(Value::Null);
        }
        let text = value.as_string().map_err(jdbc_error)?;
        return Ok(match name.to_ascii_lowercase().as_str() {
            "uuid" => Value::Uuid(text.parse().map_err(jdbc_error)?),
            "json" | "jsonb" => {
                Value::from(serde_json::from_str::<serde_json::Value>(&text).map_err(jdbc_error)?)
            }
            // rsql has no offset time type; retain the offset in the textual representation.
            _ => Value::String(text),
        });
    }
    let method = match column_type {
        JdbcType::Bit | JdbcType::Boolean => "getBoolean(I)Z",
        JdbcType::TinyInt | JdbcType::Integer | JdbcType::SmallInt => "getInt(I)I",
        JdbcType::BigInt => "getLong(I)J",
        JdbcType::Real => "getFloat(I)F",
        JdbcType::Float | JdbcType::Double => "getDouble(I)D",
        JdbcType::LongVarBinary | JdbcType::VarBinary | JdbcType::Binary | JdbcType::Blob => {
            "getBytes(I)[B"
        }
        _ => "getString(I)Ljava/lang/String;",
    };
    let value = invoke(vm, result, method, &arguments).await?;
    if invoke(vm, result, "wasNull()Z", &[])
        .await?
        .as_bool()
        .map_err(jdbc_error)?
    {
        return Ok(Value::Null);
    }
    Ok(match column_type {
        JdbcType::Bit | JdbcType::Boolean => Value::Bool(value.as_bool().map_err(jdbc_error)?),
        JdbcType::TinyInt => Value::I8(i8::try_from(value.as_i32().map_err(jdbc_error)?)?),
        JdbcType::SmallInt => Value::I16(i16::try_from(value.as_i32().map_err(jdbc_error)?)?),
        JdbcType::Integer => Value::I32(value.as_i32().map_err(jdbc_error)?),
        JdbcType::BigInt => Value::I64(value.as_i64().map_err(jdbc_error)?),
        JdbcType::Real => Value::F32(value.as_f32().map_err(jdbc_error)?),
        JdbcType::Float | JdbcType::Double => Value::F64(value.as_f64().map_err(jdbc_error)?),
        JdbcType::LongVarBinary | JdbcType::VarBinary | JdbcType::Binary | JdbcType::Blob => {
            Value::Bytes(
                value
                    .as_byte_vec_ref()
                    .map_err(jdbc_error)?
                    .iter()
                    .map(|byte| u8::from_ne_bytes(byte.to_ne_bytes()))
                    .collect(),
            )
        }
        _ => {
            let text = value.as_string().map_err(jdbc_error)?;
            match column_type {
                JdbcType::Numeric | JdbcType::Decimal => {
                    rust_decimal::Decimal::from_str_exact(&text)
                        .map_or_else(|_| Value::String(text), Value::Decimal)
                }
                JdbcType::Date => Value::Date(text.parse()?),
                JdbcType::Time => Value::Time(text.parse()?),
                JdbcType::Timestamp => Value::DateTime(text.parse()?),
                _ => Value::String(text),
            }
        }
    })
}

async fn read_array(vm: &VM, result: &JavaValue, arguments: &[JavaValue]) -> Result<Value> {
    let array = invoke(vm, result, "getArray(I)Ljava/sql/Array;", arguments).await?;
    if array.is_null() {
        return Ok(Value::Null);
    }
    let values: Result<Vec<Value>> = async {
        let result_set = invoke(vm, &array, "getResultSet()Ljava/sql/ResultSet;", &[]).await?;
        let values = Box::pin(read_result(vm, &result_set)).await;
        let close = invoke(vm, &result_set, "close()V", &[]).await;
        let mut values = values?;
        close?;
        let mut elements = Vec::new();
        while let Some(row) = values.next().await {
            elements.push(
                row.get(1)
                    .cloned()
                    .ok_or_else(|| jdbc_error("JDBC array element is missing"))?,
            );
        }
        Ok(elements)
    }
    .await;
    let free = invoke(vm, &array, "free()V", &[]).await;
    let values = values?;
    free?;
    Ok(Value::Array(values))
}
