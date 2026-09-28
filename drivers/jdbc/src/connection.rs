use crate::Options;
use crate::jdbc_type::JdbcType;
use crate::results::read_result;
use async_trait::async_trait;
use ristretto_vm::{ClassPath, ConfigurationBuilder, Reference, VM, Value as JavaValue};
use rsql_driver::{Error, QueryResult, Result, StatementMetadata, ToSql, Value};
use sqlparser::ast::Statement;
use std::fmt::Display;
use std::sync::Arc;

pub(crate) fn jdbc_error(error: impl Display) -> Error {
    Error::IoError(format!("JDBC: {error}"))
}

pub(crate) fn vm_error(error: ristretto_vm::Error) -> Error {
    if let ristretto_vm::Error::Throwable(value) = &error
        && let Ok(object) = value.as_object_ref()
        && let Ok(message) = object.value("detailMessage")
        && let Ok(message) = message.as_string()
    {
        return jdbc_error(format!("{}: {message}", object.class().name()));
    }
    jdbc_error(error)
}

/// A JDBC connection and its embedded JVM.
#[derive(Debug)]
pub struct Connection {
    url: String,
    is_h2: bool,
    pub(crate) vm: Arc<VM>,
    pub(crate) inner: JavaValue,
}

impl Connection {
    /// Connect using explicit options (also used by database-specific wrappers).
    ///
    /// # Errors
    /// Returns an error if JVM initialization, driver loading, or connection fails.
    pub async fn new(url: &str, options: Options) -> Result<Self> {
        local_future(Self::connect(url, options)).await
    }

    /// Returns whether the underlying JDBC connection is closed.
    ///
    /// # Errors
    /// Returns an error if the JVM cannot determine the connection state.
    pub async fn is_closed(&self) -> Result<bool> {
        local_future(async {
            invoke(&self.vm, &self.inner, "isClosed()Z", &[])
                .await?
                .as_bool()
                .map_err(jdbc_error)
        })
        .await
    }

    async fn connect(url: &str, options: Options) -> Result<Self> {
        #[cfg(not(target_family = "wasm"))]
        let mut classpath = crate::cache::driver_classpath(&options.dependencies).await?;
        #[cfg(target_family = "wasm")]
        let mut classpath = {
            if !options.dependencies.is_empty() {
                return Err(jdbc_error(
                    "Maven downloads are unavailable on WebAssembly; supply preloaded JARs with classpath",
                ));
            }
            Vec::new()
        };
        classpath.extend(options.classpath);
        let paths: Vec<String> = classpath
            .iter()
            .map(|path| {
                path.to_str()
                    .map(str::to_owned)
                    .ok_or_else(|| jdbc_error("Classpath must be valid UTF-8"))
            })
            .collect::<Result<_>>()?;
        let mut configuration = ConfigurationBuilder::new();
        if !paths.is_empty() {
            configuration = configuration.class_path(ClassPath::from(paths.as_slice()));
        }
        let vm = VM::new(configuration.build().map_err(jdbc_error)?)
            .await
            .map_err(vm_error)?;
        let jdbc_url = java_string(&vm, &options.url).await?;
        let properties = vm
            .object("java.util.Properties", "", &[] as &[JavaValue])
            .await
            .map_err(vm_error)?;
        let connection = if let Some(driver) = options.driver {
            let driver = vm
                .object(driver, "", &[] as &[JavaValue])
                .await
                .map_err(vm_error)?;
            invoke(
                &vm,
                &driver,
                "connect(Ljava/lang/String;Ljava/util/Properties;)Ljava/sql/Connection;",
                &[jdbc_url, properties],
            )
            .await?
        } else {
            vm.try_invoke(
                "java.sql.DriverManager",
                "getConnection(Ljava/lang/String;Ljava/util/Properties;)Ljava/sql/Connection;",
                &[jdbc_url, properties],
            )
            .await
            .map_err(vm_error)?
        };
        if connection.is_null() {
            return Err(jdbc_error("Driver does not accept the JDBC URL"));
        }
        Ok(Self {
            url: url.into(),
            is_h2: options.url.starts_with("jdbc:h2:"),
            vm,
            inner: connection,
        })
    }

    async fn prepare(&self, sql: &str, params: &[&dyn ToSql]) -> Result<JavaValue> {
        if self.is_closed().await? {
            return Err(jdbc_error("Connection is closed"));
        }
        let sql = java_string(&self.vm, sql).await?;
        let statement = invoke(
            &self.vm,
            &self.inner,
            "prepareStatement(Ljava/lang/String;)Ljava/sql/PreparedStatement;",
            &[sql],
        )
        .await?;
        for (index, value) in rsql_driver::to_values(params).iter().enumerate() {
            if let Err(error) = self
                .bind(&statement, i32::try_from(index + 1)?, value)
                .await
            {
                let _ = invoke(&self.vm, &statement, "close()V", &[]).await;
                return Err(error);
            }
        }
        Ok(statement)
    }

    async fn bind(&self, statement: &JavaValue, index: i32, value: &Value) -> Result<()> {
        if let Value::Array(values) = value
            && !self.is_h2
        {
            return crate::values::bind_array(&self.vm, &self.inner, statement, index, values)
                .await;
        }
        if matches!(value, Value::Uuid(_)) || (!self.is_h2 && matches!(value, Value::Map(_))) {
            let object = crate::values::boxed_value(&self.vm, value).await?;
            invoke(
                &self.vm,
                statement,
                "setObject(ILjava/lang/Object;I)V",
                &[
                    JavaValue::Int(index),
                    object,
                    JavaValue::Int(JdbcType::Other as i32),
                ],
            )
            .await?;
            return Ok(());
        }
        let (method, value) = match value {
            Value::Null => ("setNull(II)V", JavaValue::Int(JdbcType::Null as i32)),
            Value::Bool(value) => ("setBoolean(IZ)V", JavaValue::Int(i32::from(*value))),
            Value::I8(value) => ("setByte(IB)V", JavaValue::Int(i32::from(*value))),
            Value::I16(value) => ("setShort(IS)V", JavaValue::Int(i32::from(*value))),
            Value::I32(value) => ("setInt(II)V", JavaValue::Int(*value)),
            Value::I64(value) => ("setLong(IJ)V", JavaValue::Long(*value)),
            Value::U8(value) => ("setShort(IS)V", JavaValue::Int(i32::from(*value))),
            Value::U16(value) => ("setInt(II)V", JavaValue::Int(i32::from(*value))),
            Value::U32(value) => ("setLong(IJ)V", JavaValue::Long(i64::from(*value))),
            Value::F32(value) => ("setFloat(IF)V", JavaValue::Float(*value)),
            Value::F64(value) => ("setDouble(ID)V", JavaValue::Double(*value)),
            Value::Bytes(value) => (
                "setBytes(I[B)V",
                JavaValue::new_object(self.vm.garbage_collector(), Reference::from(value.clone())),
            ),
            // H2 interprets UTF-8 bytes as a JSON document, but strings as JSON strings.
            Value::Map(_) => (
                "setBytes(I[B)V",
                JavaValue::new_object(
                    self.vm.garbage_collector(),
                    Reference::from(serde_json::to_vec(value).map_err(jdbc_error)?),
                ),
            ),
            // H2's setArray calls getArray, which converts LocalTime to lossy java.sql.Time.
            Value::Array(_) => (
                "setObject(ILjava/lang/Object;)V",
                crate::values::boxed_value(&self.vm, value).await?,
            ),
            Value::Decimal(_) | Value::I128(_) | Value::U64(_) | Value::U128(_) => {
                let decimal = self
                    .vm
                    .object(
                        "java.math.BigDecimal",
                        "Ljava/lang/String;",
                        &[value.to_string()],
                    )
                    .await
                    .map_err(jdbc_error)?;
                ("setBigDecimal(ILjava/math/BigDecimal;)V", decimal)
            }
            Value::Date(_) | Value::Time(_) | Value::DateTime(_) => {
                let setter = match value {
                    Value::Date(_) => "setDate(ILjava/sql/Date;)V",
                    Value::Time(_) => "setObject(ILjava/lang/Object;)V",
                    _ => "setTimestamp(ILjava/sql/Timestamp;)V",
                };
                (setter, crate::values::boxed_value(&self.vm, value).await?)
            }
            _ => (
                "setString(ILjava/lang/String;)V",
                java_string(&self.vm, &value.to_string()).await?,
            ),
        };
        invoke(&self.vm, statement, method, &[JavaValue::Int(index), value]).await?;
        Ok(())
    }
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    fn match_statement(&self, statement: &Statement) -> StatementMetadata {
        if self.is_h2
            && matches!(
                statement,
                Statement::ShowTables { .. }
                    | Statement::ShowSchemas { .. }
                    | Statement::ShowColumns { .. }
                    | Statement::ShowVariable { .. }
                    | Statement::Call(_)
            )
        {
            StatementMetadata::Query
        } else {
            self.default_match_statement(statement)
        }
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        local_future(async {
            let statement = self.prepare(sql, params).await?;
            let result: Result<u64> = async {
                let count = invoke(&self.vm, &statement, "executeUpdate()I", &[])
                    .await?
                    .as_i32()
                    .map_err(jdbc_error)?;
                Ok(u64::try_from(count)?)
            }
            .await;
            let close_result = invoke(&self.vm, &statement, "close()V", &[]).await;
            let count = result?;
            close_result?;
            Ok(count)
        })
        .await
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        local_future(async {
            let statement = self.prepare(sql, params).await?;
            let result = async {
                let result_set = invoke(
                    &self.vm,
                    &statement,
                    "executeQuery()Ljava/sql/ResultSet;",
                    &[],
                )
                .await?;
                read_result(&self.vm, &result_set).await
            }
            .await;
            // Closing a statement also closes its result set, including on conversion errors.
            let close_result = invoke(&self.vm, &statement, "close()V", &[]).await;
            let result = result?;
            close_result?;
            Ok(Box::new(result) as Box<dyn QueryResult>)
        })
        .await
    }

    async fn close(&mut self) -> Result<()> {
        local_future(async {
            if !self.is_closed().await? {
                invoke(&self.vm, &self.inner, "close()V", &[]).await?;
            }
            Ok(())
        })
        .await
    }

    async fn metadata(&mut self) -> Result<rsql_driver::Metadata> {
        local_future(async {
            if self.is_closed().await? {
                return Err(jdbc_error("Connection is closed"));
            }
            crate::metadata::metadata(self).await
        })
        .await
    }
}

pub(crate) async fn java_string(vm: &VM, text: &str) -> Result<JavaValue> {
    vm.try_invoke(
        "java.lang.String",
        "valueOf(Ljava/lang/Object;)Ljava/lang/String;",
        &[text],
    )
    .await
    .map_err(vm_error)
}

/// Resolve instance methods against the concrete class (including inherited methods).
pub(crate) async fn invoke(
    vm: &VM,
    object: &JavaValue,
    method: &str,
    arguments: &[JavaValue],
) -> Result<JavaValue> {
    let mut class = object.as_object_ref().map_err(jdbc_error)?.class().clone();
    let (name, descriptor) = method
        .split_once('(')
        .ok_or_else(|| jdbc_error("Invalid method signature"))?;
    let descriptor = format!("({descriptor}");
    while class.method(name, &descriptor).is_none() {
        class = class
            .parent()
            .map_err(jdbc_error)?
            .ok_or_else(|| jdbc_error(format!("Method {method} not found")))?;
    }
    let mut parameters = Vec::with_capacity(arguments.len() + 1);
    parameters.push(object.clone());
    parameters.extend_from_slice(arguments);
    Ok(vm
        .invoke(class.name(), method, &parameters)
        .await
        .map_err(vm_error)?
        .unwrap_or(JavaValue::Object(None)))
}

// Ristretto uses non-Send futures on WASM. Keep rsql’s Send interface while enforcing
// that these futures are polled and dropped on the thread that created them.
#[cfg(target_family = "wasm")]
fn local_future<F: Future>(future: F) -> send_wrapper::SendWrapper<F> {
    send_wrapper::SendWrapper::new(future)
}

#[cfg(not(target_family = "wasm"))]
fn local_future<F: Future>(future: F) -> F {
    future
}
