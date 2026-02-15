use crate::metadata;
use crate::results::PolarsQueryResult;
use async_trait::async_trait;
use polars_sql::SQLContext;
use rsql_driver::Error::{InvalidUrl, IoError};
use rsql_driver::{Metadata, QueryResult, Result, ToSql, Value};
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Connection for drivers based on Polars `SQLContext`
pub struct Connection {
    url: String,
    context: Arc<Mutex<SQLContext>>,
}

impl Connection {
    /// Create a new `Connection`
    ///
    /// # Errors
    /// if the URL is invalid
    #[expect(clippy::unused_async)]
    pub async fn new(url: &str, context: SQLContext) -> Result<Self> {
        Ok(Self {
            url: url.to_string(),
            context: Arc::new(Mutex::new(context)),
        })
    }
}

impl Connection {
    /// Get the `SQLContext`
    pub(crate) fn context(&self) -> Arc<Mutex<SQLContext>> {
        self.context.clone()
    }
}

/// Get the table name from the file name
///
/// # Errors
/// if the file name is invalid
pub fn get_table_name<S: AsRef<str>>(file_name: S) -> Result<String> {
    let file_name = file_name.as_ref();
    let file_name = Path::new(file_name)
        .file_name()
        .ok_or(InvalidUrl("Invalid file name".to_string()))?
        .to_str()
        .ok_or(InvalidUrl("Invalid file name".to_string()))?;
    let table_name = file_name.split('.').next().unwrap_or(file_name);
    Ok(table_name.to_string())
}

#[async_trait]
impl rsql_driver::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let sql = substitute_params(sql, params);
        let mut context = self.context.lock().await;
        let result = context
            .execute(&sql)
            .map_err(|error| IoError(error.to_string()))?;
        let data_frame = result
            .collect()
            .map_err(|error| IoError(error.to_string()))?;
        let rows =
            u64::try_from(data_frame.height()).map_err(|error| IoError(error.to_string()))?;
        Ok(rows)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let sql = substitute_params(sql, params);
        let mut context = self.context.lock().await;
        let result = context
            .execute(&sql)
            .map_err(|error| IoError(error.to_string()))?;
        let data_frame = result
            .collect()
            .map_err(|error| IoError(error.to_string()))?;
        let columns = data_frame
            .get_column_names()
            .iter()
            .map(ToString::to_string)
            .collect();

        let query_result = PolarsQueryResult::new(columns, data_frame);
        Ok(Box::new(query_result))
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        metadata::get_metadata(self).await
    }
}

/// Substitute `?` placeholders with inline values for engines without native bind support.
fn substitute_params(sql: &str, params: &[&dyn ToSql]) -> String {
    if params.is_empty() {
        return sql.to_string();
    }
    let values = rsql_driver::to_values(params);
    let mut result = String::with_capacity(sql.len());
    let mut param_index = 0;
    let mut chars = sql.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                result.push(ch);
                for ch in chars.by_ref() {
                    result.push(ch);
                    if ch == '\'' {
                        break;
                    }
                }
            }
            '"' => {
                result.push(ch);
                for ch in chars.by_ref() {
                    result.push(ch);
                    if ch == '"' {
                        break;
                    }
                }
            }
            '?' => {
                if param_index < values.len() {
                    result.push_str(&format_value_inline(&values[param_index]));
                    param_index += 1;
                } else {
                    result.push(ch);
                }
            }
            _ => result.push(ch),
        }
    }
    result
}

fn format_value_inline(value: &Value) -> String {
    match value {
        Value::Null => "NULL".to_string(),
        Value::Bool(v) => if *v { "TRUE" } else { "FALSE" }.to_string(),
        Value::String(v) => format!("'{}'", v.replace('\'', "''")),
        Value::Bytes(v) => format!(
            "X'{}'",
            v.iter().map(|b| format!("{b:02x}")).collect::<String>()
        ),
        _ => value.to_string(),
    }
}

#[expect(clippy::missing_fields_in_debug)]
impl Debug for Connection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("url", &self.url)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use polars::prelude::*;
    use rsql_driver::{Connection, Value};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_connection() -> Result<()> {
        let ids = Series::new("id".into(), &[1i64, 2i64]);
        let names = Series::new("name".into(), &["John Doe", "Jane Smith"]);
        let data_frame = DataFrame::new_infer_height(vec![Column::from(ids), Column::from(names)])
            .map_err(|error| IoError(error.to_string()))?;
        let context = SQLContext::new();
        context.register("users", data_frame.lazy());
        let mut connection = super::Connection::new("polars://", context).await?;

        let mut query_result = connection
            .query("SELECT id, name FROM users ORDER BY id", &[])
            .await?;

        assert_eq!(query_result.columns(), vec!["id", "name"]);
        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
        );
        assert_eq!(
            query_result.next().await.cloned(),
            Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
        );
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_query_with_params() -> Result<()> {
        let ids = Series::new("id".into(), &[1i64, 2i64, 3i64]);
        let names = Series::new("name".into(), &["Alice", "Bob", "Charlie"]);
        let data_frame = DataFrame::new_infer_height(vec![Column::from(ids), Column::from(names)])
            .map_err(|error| IoError(error.to_string()))?;
        let context = SQLContext::new();
        context.register("users", data_frame.lazy());
        let mut connection = super::Connection::new("polars://", context).await?;

        let id_param: i64 = 2;
        let mut query_result = connection
            .query("SELECT id, name FROM users WHERE id = ?", &[&id_param])
            .await?;

        assert_eq!(query_result.columns(), vec!["id", "name"]);
        let row = query_result.next().await;
        assert!(row.is_some());
        let row = row.unwrap();
        assert_eq!(row[0], Value::I64(2));
        assert!(query_result.next().await.is_none());

        connection.close().await?;
        Ok(())
    }

    #[test]
    fn test_substitute_params_empty() {
        let sql = "SELECT * FROM users";
        let result = substitute_params(sql, &[]);
        assert_eq!(result, "SELECT * FROM users");
    }

    #[test]
    fn test_substitute_params_with_values() {
        let id: i32 = 42;
        let name = "Alice";
        let result = substitute_params(
            "SELECT * FROM users WHERE id = ? AND name = ?",
            &[&id, &name],
        );
        assert_eq!(
            result,
            "SELECT * FROM users WHERE id = 42 AND name = 'Alice'"
        );
    }

    #[test]
    fn test_substitute_params_preserves_string_literal() {
        let id: i32 = 1;
        let result = substitute_params("SELECT * FROM t WHERE x = '?' AND id = ?", &[&id]);
        assert_eq!(result, "SELECT * FROM t WHERE x = '?' AND id = 1");
    }

    #[test]
    fn test_substitute_params_null() {
        let val: Option<String> = None;
        let result = substitute_params("SELECT * FROM t WHERE x = ?", &[&val]);
        assert_eq!(result, "SELECT * FROM t WHERE x = NULL");
    }
}
