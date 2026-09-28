use crate::connection::{invoke, java_string, jdbc_error};
use ristretto_vm::{Reference, VM, Value as JavaValue};
use rsql_driver::{Result, Value};

pub(crate) async fn bind_array(
    vm: &VM,
    connection: &JavaValue,
    statement: &JavaValue,
    index: i32,
    values: &[Value],
) -> Result<()> {
    let metadata = invoke(
        vm,
        statement,
        "getParameterMetaData()Ljava/sql/ParameterMetaData;",
        &[],
    )
    .await?;
    let name = invoke(
        vm,
        &metadata,
        "getParameterTypeName(I)Ljava/lang/String;",
        &[JavaValue::Int(index)],
    )
    .await?
    .as_string()
    .map_err(jdbc_error)?;
    // PostgreSQL reports _int4; other drivers report INTEGER ARRAY or INTEGER[].
    let base = name
        .strip_prefix('_')
        .or_else(|| name.strip_suffix(" ARRAY"))
        .or_else(|| name.strip_suffix("[]"))
        .unwrap_or(&name);
    let name = java_string(vm, base).await?;
    let objects = object_array(vm, values).await?;
    let array = invoke(
        vm,
        connection,
        "createArrayOf(Ljava/lang/String;[Ljava/lang/Object;)Ljava/sql/Array;",
        &[name, objects],
    )
    .await?;
    invoke(
        vm,
        statement,
        "setArray(ILjava/sql/Array;)V",
        &[JavaValue::Int(index), array],
    )
    .await?;
    Ok(())
}

async fn object_array(vm: &VM, values: &[Value]) -> Result<JavaValue> {
    let mut objects = Vec::with_capacity(values.len());
    let mut component = None;
    for value in values {
        let object = Box::pin(boxed_value(vm, value)).await?;
        if !matches!(object, JavaValue::Object(None)) {
            let name = object
                .as_reference()
                .map_err(jdbc_error)?
                .class_name()
                .map_err(jdbc_error)?;
            component = Some(match component {
                None => name,
                Some(previous) if previous == name => previous,
                Some(_) => "java/lang/Object".to_owned(),
            });
        }
        objects.push(object);
    }
    // Preserve homogeneous element types, including byte[][] for SQL binary arrays.
    let component = component.as_deref().unwrap_or("java/lang/Object");
    let name = if component.starts_with('[') {
        format!("[{component}")
    } else {
        format!("[L{component};")
    };
    let class = vm.class(&name).await.map_err(jdbc_error)?;
    let reference = Reference::try_from((class, objects)).map_err(jdbc_error)?;
    Ok(JavaValue::new_object(vm.garbage_collector(), reference))
}

pub(crate) async fn boxed_value(vm: &VM, value: &Value) -> Result<JavaValue> {
    let (class, descriptor, argument) = match value {
        Value::Null => return Ok(JavaValue::Object(None)),
        Value::Bool(value) => ("java.lang.Boolean", "Z", JavaValue::Int(i32::from(*value))),
        Value::I8(value) => ("java.lang.Byte", "B", JavaValue::Int(i32::from(*value))),
        Value::I16(value) => ("java.lang.Short", "S", JavaValue::Int(i32::from(*value))),
        Value::I32(value) => ("java.lang.Integer", "I", JavaValue::Int(*value)),
        Value::I64(value) => ("java.lang.Long", "J", JavaValue::Long(*value)),
        Value::U8(value) => ("java.lang.Short", "S", JavaValue::Int(i32::from(*value))),
        Value::U16(value) => ("java.lang.Integer", "I", JavaValue::Int(i32::from(*value))),
        Value::U32(value) => ("java.lang.Long", "J", JavaValue::Long(i64::from(*value))),
        Value::F32(value) => ("java.lang.Float", "F", JavaValue::Float(*value)),
        Value::F64(value) => ("java.lang.Double", "D", JavaValue::Double(*value)),
        Value::Bytes(value) => {
            return Ok(JavaValue::new_object(
                vm.garbage_collector(),
                Reference::from(value.clone()),
            ));
        }
        Value::Array(values) => return Box::pin(object_array(vm, values)).await,
        Value::String(value) => return java_string(vm, value).await,
        Value::Map(_) => {
            return java_string(vm, &serde_json::to_string(value).map_err(jdbc_error)?).await;
        }
        Value::Uuid(value) => {
            return vm
                .try_invoke(
                    "java.util.UUID",
                    "fromString(Ljava/lang/String;)Ljava/util/UUID;",
                    &[value.to_string()],
                )
                .await
                .map_err(jdbc_error);
        }
        Value::Date(_) | Value::Time(_) | Value::DateTime(_) => {
            let (class, method) = match value {
                Value::Date(_) => (
                    "java.sql.Date",
                    "valueOf(Ljava/lang/String;)Ljava/sql/Date;",
                ),
                Value::Time(_) => (
                    "java.time.LocalTime",
                    "parse(Ljava/lang/CharSequence;)Ljava/time/LocalTime;",
                ),
                _ => (
                    "java.sql.Timestamp",
                    "valueOf(Ljava/lang/String;)Ljava/sql/Timestamp;",
                ),
            };
            return vm
                .try_invoke(class, method, &[value.to_string().replace('T', " ")])
                .await
                .map_err(jdbc_error);
        }
        Value::Decimal(_) | Value::I128(_) | Value::U64(_) | Value::U128(_) => (
            "java.math.BigDecimal",
            "Ljava/lang/String;",
            java_string(vm, &value.to_string()).await?,
        ),
    };
    vm.object(class, descriptor, &[argument])
        .await
        .map_err(jdbc_error)
}
