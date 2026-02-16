use crate::error::Result;
use crate::{Metadata, ToSql, Value};
use async_trait::async_trait;
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, GenericDialect};
use sqlparser::parser::Parser;

use i18n_inflector::language_rules;
use jiff::civil::DateTime;
use jiff::tz::Offset;
use jiff::{Timestamp, ToSpan};
use std::fmt::Debug;

/// A single row of a query result
pub type Row = Vec<Value>;

/// Convert `?` placeholders to numbered `$1, $2, ...` placeholders.
/// Used by PostgreSQL-family drivers.
#[must_use]
pub fn convert_to_numbered_placeholders(sql: &str) -> String {
    convert_question_marks(sql, "$")
}

/// Convert `?` placeholders to numbered `@P1, @P2, ...` placeholders.
/// Used by SQL Server (tiberius) drivers.
#[must_use]
pub fn convert_to_at_placeholders(sql: &str) -> String {
    convert_question_marks(sql, "@P")
}

fn convert_question_marks(sql: &str, prefix: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let mut param_index = 0u32;
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
                param_index += 1;
                result.push_str(prefix);
                result.push_str(&param_index.to_string());
            }
            _ => result.push(ch),
        }
    }
    result
}

/// Results from a query
#[async_trait]
pub trait QueryResult: Debug + Send + Sync {
    fn columns(&self) -> &[String];
    async fn next(&mut self) -> Option<&Row>;
}

/// Query result with a limit
#[derive(Debug)]
pub struct LimitQueryResult {
    inner: Box<dyn QueryResult>,
    row_index: usize,
    limit: usize,
}

impl LimitQueryResult {
    #[must_use]
    pub fn new(inner: Box<dyn QueryResult>, limit: usize) -> Self {
        Self {
            inner,
            row_index: 0,
            limit,
        }
    }
}

#[async_trait]
impl QueryResult for LimitQueryResult {
    fn columns(&self) -> &[String] {
        self.inner.columns()
    }

    async fn next(&mut self) -> Option<&Row> {
        if self.row_index >= self.limit {
            return None;
        }

        let value = self.inner.next().await;
        self.row_index += 1;
        value
    }
}

/// In-memory query result
#[derive(Clone, Debug, Default)]
pub struct MemoryQueryResult {
    columns: Vec<String>,
    row_index: usize,
    rows: Vec<Row>,
}

impl MemoryQueryResult {
    #[must_use]
    pub fn new(columns: Vec<String>, rows: Vec<Row>) -> Self {
        Self {
            columns,
            row_index: 0,
            rows,
        }
    }
}

#[async_trait]
impl QueryResult for MemoryQueryResult {
    fn columns(&self) -> &[String] {
        &self.columns
    }

    async fn next(&mut self) -> Option<&Row> {
        if self.row_index >= self.rows.len() {
            return None;
        }
        let row = &self.rows[self.row_index];
        self.row_index += 1;
        Some(row)
    }
}

#[derive(Debug, Clone)]
pub enum StatementMetadata {
    DDL,
    DML,
    Query,
    Unknown,
}

/// Connection to a database
#[async_trait]
pub trait Connection: Debug + Send + Sync {
    fn url(&self) -> &String;
    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64>;
    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>>;

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        let dialect = self.dialect();
        let metadata = Metadata::with_dialect(dialect);
        Ok(metadata)
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(GenericDialect)
    }

    fn parse_sql(&self, sql: &str) -> StatementMetadata {
        let statements = Parser::parse_sql(self.dialect().as_ref(), sql).unwrap_or_default();

        if let Some(statement) = statements.first() {
            self.match_statement(statement)
        } else {
            let command = if sql.len() > 6 { &sql[..6] } else { "" };
            if command.to_lowercase() == "select" {
                StatementMetadata::Query
            } else {
                StatementMetadata::Unknown
            }
        }
    }

    fn default_match_statement(&self, statement: &Statement) -> StatementMetadata {
        match statement {
            Statement::CreateSchema { .. }
            | Statement::CreateDatabase { .. }
            | Statement::CreateView { .. }
            | Statement::CreateIndex(_)
            | Statement::CreateTable(_)
            | Statement::CreateSequence { .. }
            | Statement::AlterTable { .. }
            | Statement::AlterIndex { .. }
            | Statement::Drop { .. } => StatementMetadata::DDL,
            Statement::Query(_) => StatementMetadata::Query,
            Statement::Insert(_) | Statement::Update { .. } | Statement::Delete(_) => {
                StatementMetadata::DML
            }
            _ => StatementMetadata::Unknown,
        }
    }

    fn match_statement(&self, statement: &Statement) -> StatementMetadata {
        self.default_match_statement(statement)
    }
}

#[derive(Debug)]
pub struct CachedMetadataConnection {
    connection: Box<dyn Connection>,
    locale: String,
    metadata: Option<Metadata>,
    timestamp: DateTime,
}

impl CachedMetadataConnection {
    #[must_use]
    pub fn new(connection: Box<dyn Connection>) -> Self {
        let now = Offset::UTC.to_datetime(Timestamp::now());
        Self {
            connection,
            locale: "en".to_string(),
            metadata: None,
            timestamp: now,
        }
    }

    /// Sets the locale used for table name inference (singularization/pluralization).
    pub fn set_locale<S: Into<String>>(&mut self, locale: S) {
        self.locale = locale.into();
    }
}

#[async_trait]
impl Connection for CachedMetadataConnection {
    fn url(&self) -> &String {
        self.connection.url()
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        if let StatementMetadata::DDL = self.parse_sql(sql) {
            self.metadata = None;
        }

        self.connection.execute(sql, params).await
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        let now = Offset::UTC.to_datetime(Timestamp::now());
        let timestamp = self.timestamp.checked_add(1.minute())?;

        if timestamp < now {
            self.metadata = None;
        }

        if let Some(metadata) = &self.metadata {
            Ok(metadata.clone())
        } else {
            let mut metadata = self.connection.metadata().await?;
            let language_rules = match language_rules(&self.locale) {
                Ok(rules) => rules,
                Err(_) => language_rules("en")?,
            };

            metadata.infer_primary_keys(language_rules);
            metadata.infer_foreign_keys(language_rules);
            self.metadata = Some(metadata.clone());
            Ok(metadata)
        }
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        self.connection.query(sql, params).await
    }

    async fn close(&mut self) -> Result<()> {
        self.connection.close().await
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        self.connection.dialect()
    }

    fn match_statement(&self, statement: &Statement) -> StatementMetadata {
        self.connection.match_statement(statement)
    }
}

type MockExecuteFn = Box<dyn FnMut(&str, &[Value]) -> Result<u64> + Send + Sync>;
type MockQueryFn = Box<dyn FnMut(&str, &[Value]) -> Result<Box<dyn QueryResult>> + Send + Sync>;
type MockParseSqlFn = Box<dyn FnMut(&str) -> StatementMetadata + Send + Sync>;

/// A mock implementation of [`Connection`] for testing.
///
/// Supports setting expectations via `expect_*` methods with `.returning()` closures.
/// Execute and query expectations optionally support `.with()` for SQL matching.
pub struct MockConnection {
    url: String,
    execute_fn: Option<MockExecuteFn>,
    execute_sql: Option<String>,
    query_fn: Option<MockQueryFn>,
    close_fn: Option<Box<dyn FnMut() -> Result<()> + Send + Sync>>,
    metadata_fn: Option<Box<dyn FnMut() -> Result<Metadata> + Send + Sync>>,
    parse_sql_fn: std::sync::Mutex<Option<MockParseSqlFn>>,
}

impl Debug for MockConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockConnection").finish()
    }
}

/// Builder for setting an execute expectation on [`MockConnection`].
pub struct MockExecuteExpectation<'a> {
    mock: &'a mut MockConnection,
    sql: Option<String>,
}

impl MockExecuteExpectation<'_> {
    /// Restrict this expectation to calls matching the given SQL and empty params.
    #[must_use]
    pub fn with(mut self, sql: &str, _params: Vec<Value>) -> Self {
        self.sql = Some(sql.to_string());
        self
    }

    /// Set the closure to call when the expectation is matched.
    pub fn returning<F>(self, f: F)
    where
        F: FnMut(&str, &[Value]) -> Result<u64> + Send + Sync + 'static,
    {
        self.mock.execute_fn = Some(Box::new(f));
        self.mock.execute_sql = self.sql;
    }
}

/// Builder for setting a query expectation on [`MockConnection`].
pub struct MockQueryExpectation<'a> {
    mock: &'a mut MockConnection,
}

impl MockQueryExpectation<'_> {
    /// Set the closure to call when the expectation is matched.
    pub fn returning<F>(self, f: F)
    where
        F: FnMut(&str, &[Value]) -> Result<Box<dyn QueryResult>> + Send + Sync + 'static,
    {
        self.mock.query_fn = Some(Box::new(f));
    }
}

/// Builder for setting a close expectation on [`MockConnection`].
pub struct MockCloseExpectation<'a> {
    mock: &'a mut MockConnection,
}

impl MockCloseExpectation<'_> {
    /// Set the closure to call when the expectation is matched.
    pub fn returning<F>(self, f: F)
    where
        F: FnMut() -> Result<()> + Send + Sync + 'static,
    {
        self.mock.close_fn = Some(Box::new(f));
    }
}

/// Builder for setting a metadata expectation on [`MockConnection`].
pub struct MockMetadataExpectation<'a> {
    mock: &'a mut MockConnection,
}

impl MockMetadataExpectation<'_> {
    /// Ignored; provided for API compatibility.
    #[must_use]
    pub fn with(self) -> Self {
        self
    }

    /// Set the closure to call when the expectation is matched.
    pub fn returning<F>(self, f: F)
    where
        F: FnMut() -> Result<Metadata> + Send + Sync + 'static,
    {
        self.mock.metadata_fn = Some(Box::new(f));
    }
}

/// Builder for setting a parse_sql expectation on [`MockConnection`].
pub struct MockParseSqlExpectation<'a> {
    mock: &'a mut MockConnection,
}

impl MockParseSqlExpectation<'_> {
    /// Ignored; provided for API compatibility.
    #[must_use]
    pub fn with(self, _sql: impl Into<String>) -> Self {
        self
    }

    /// Set the closure to call when the expectation is matched.
    pub fn returning<F>(self, f: F)
    where
        F: FnMut(&str) -> StatementMetadata + Send + Sync + 'static,
    {
        *self.mock.parse_sql_fn.lock().unwrap() = Some(Box::new(f));
    }
}

impl MockConnection {
    /// Create a new mock with no expectations set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            url: String::new(),
            execute_fn: None,
            execute_sql: None,
            query_fn: None,
            close_fn: None,
            metadata_fn: None,
            parse_sql_fn: std::sync::Mutex::new(None),
        }
    }

    /// Set an expectation for [`Connection::execute`].
    pub fn expect_execute(&mut self) -> MockExecuteExpectation<'_> {
        MockExecuteExpectation {
            mock: self,
            sql: None,
        }
    }

    /// Set an expectation for [`Connection::query`].
    pub fn expect_query(&mut self) -> MockQueryExpectation<'_> {
        MockQueryExpectation { mock: self }
    }

    /// Set an expectation for [`Connection::close`].
    pub fn expect_close(&mut self) -> MockCloseExpectation<'_> {
        MockCloseExpectation { mock: self }
    }

    /// Set an expectation for [`Connection::metadata`].
    pub fn expect_metadata(&mut self) -> MockMetadataExpectation<'_> {
        MockMetadataExpectation { mock: self }
    }

    /// Set an expectation for [`Connection::parse_sql`].
    pub fn expect_parse_sql(&mut self) -> MockParseSqlExpectation<'_> {
        MockParseSqlExpectation { mock: self }
    }
}

impl Default for MockConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Connection for MockConnection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<u64> {
        let values: Vec<Value> = params.iter().map(|p| p.to_value()).collect();
        if let Some(expected_sql) = &self.execute_sql {
            assert_eq!(sql, expected_sql, "MockConnection: unexpected SQL");
        }
        let f = self
            .execute_fn
            .as_mut()
            .expect("MockConnection: execute called without expectation");
        f(sql, &values)
    }

    async fn query(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Box<dyn QueryResult>> {
        let values: Vec<Value> = params.iter().map(|p| p.to_value()).collect();
        let f = self
            .query_fn
            .as_mut()
            .expect("MockConnection: query called without expectation");
        f(sql, &values)
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(f) = self.close_fn.as_mut() {
            f()
        } else {
            Ok(())
        }
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        if let Some(f) = self.metadata_fn.as_mut() {
            f()
        } else {
            let dialect = self.dialect();
            let metadata = Metadata::with_dialect(dialect);
            Ok(metadata)
        }
    }

    fn parse_sql(&self, sql: &str) -> StatementMetadata {
        if let Some(f) = self.parse_sql_fn.lock().unwrap().as_mut() {
            return f(sql);
        }

        let statements = Parser::parse_sql(self.dialect().as_ref(), sql).unwrap_or_default();

        if let Some(statement) = statements.first() {
            self.match_statement(statement)
        } else {
            let command = if sql.len() > 6 { &sql[..6] } else { "" };
            if command.to_lowercase() == "select" {
                StatementMetadata::Query
            } else {
                StatementMetadata::Unknown
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Value;

    #[tokio::test]
    async fn test_memory_query_result_new() {
        let columns = vec!["a".to_string()];
        let rows = vec![vec![Value::String("foo".to_string())]];

        let mut result = MemoryQueryResult::new(columns, rows);

        let columns = result.columns();
        let column = columns.first().expect("no column");
        assert_eq!(column, &"a".to_string());

        let row = result.next().await.expect("no row");
        let value = row.first().expect("no value");
        assert_eq!(value, &Value::String("foo".to_string()));
    }

    #[tokio::test]
    async fn test_limit_query_result() {
        let columns = vec!["id".to_string()];
        let rows = vec![
            vec![Value::I64(1)],
            vec![Value::I64(2)],
            vec![Value::I64(3)],
            vec![Value::I64(4)],
            vec![Value::I64(5)],
        ];
        let memory_result = MemoryQueryResult::new(columns, rows);
        let mut result = LimitQueryResult::new(Box::new(memory_result), 2);

        let columns = result.columns();
        let column = columns.first().expect("no column");
        assert_eq!(column, &"id".to_string());

        let mut data: Vec<String> = Vec::new();
        while let Some(row) = result.next().await {
            let value = row.first().expect("no value");
            data.push(value.to_string());
        }

        assert_eq!(data, ["1".to_string(), "2".to_string()]);
    }

    #[tokio::test]
    async fn test_limit_query_result_limit_exceeds_rows() {
        let columns = vec!["id".to_string()];
        let rows = vec![vec![Value::I64(1)]];
        let memory_result = MemoryQueryResult::new(columns, rows);
        let mut result = LimitQueryResult::new(Box::new(memory_result), 100);

        let columns = result.columns();
        let column = columns.first().expect("no column");
        assert_eq!(column, &"id".to_string());

        let mut data: Vec<String> = Vec::new();
        while let Some(row) = result.next().await {
            let value = row.first().expect("no value");
            data.push(value.to_string());
        }

        assert_eq!(data, ["1".to_string()]);
    }

    #[derive(Debug, PartialEq)]
    struct SampleConnection {
        url: String,
    }
    #[async_trait]
    impl Connection for SampleConnection {
        fn url(&self) -> &String {
            &self.url
        }

        async fn execute(&mut self, _sql: &str, _params: &[&dyn ToSql]) -> Result<u64> {
            Ok(0)
        }

        async fn metadata(&mut self) -> Result<Metadata> {
            Ok(Metadata::default())
        }

        async fn query(
            &mut self,
            _sql: &str,
            _params: &[&dyn ToSql],
        ) -> Result<Box<dyn QueryResult>> {
            Ok(Box::new(MemoryQueryResult::new(vec![], vec![])))
        }

        async fn close(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_default_parse_sql() {
        let connection = SampleConnection {
            url: "test".to_string(),
        };
        let ddl_queries = vec![
            "CREATE TABLE users (id INT, name VARCHAR(255))",
            "ALTER TABLE products ADD COLUMN price DECIMAL(10, 2)",
            "DROP VIEW old_view",
            "CREATE INDEX idx_user_name ON users (name)",
        ];

        for query in ddl_queries {
            let result = connection.parse_sql(query);
            assert!(matches!(result, StatementMetadata::DDL));
        }

        let select_queries = vec![
            "SELECT * FROM users",
            "SELECT id, name FROM products WHERE price > 100",
            "SELECT COUNT(*) FROM orders GROUP BY status",
        ];

        for query in select_queries {
            let result = connection.parse_sql(query);
            assert!(matches!(result, StatementMetadata::Query));
        }

        let data_manipulation_queries = vec![
            "INSERT INTO users (id, name) VALUES (1, 'John')",
            "UPDATE products SET price = 99.99 WHERE id = 1",
            "DELETE FROM orders WHERE status = 'cancelled'",
        ];

        for query in data_manipulation_queries {
            let result = connection.parse_sql(query);
            assert!(matches!(result, StatementMetadata::DML));
        }

        let session_queries = vec![
            "SET SESSION timezone = 'UTC'",
            "SET TRANSACTION ISOLATION LEVEL READ COMMITTED",
            "START TRANSACTION",
            "COMMIT",
            "ROLLBACK",
        ];

        for query in session_queries {
            let result = connection.parse_sql(query);
            assert!(matches!(result, StatementMetadata::Unknown));
        }

        let invalid_queries = vec!["SELECT", "INSERT IN table", "DROP"];

        for query in invalid_queries {
            let result = connection.parse_sql(query);
            assert!(matches!(result, StatementMetadata::Unknown));
        }
    }

    #[test]
    fn test_convert_to_numbered_placeholders() {
        assert_eq!(
            convert_to_numbered_placeholders("SELECT * FROM users WHERE id = ? AND name = ?"),
            "SELECT * FROM users WHERE id = $1 AND name = $2"
        );
    }

    #[test]
    fn test_convert_to_numbered_placeholders_no_params() {
        assert_eq!(
            convert_to_numbered_placeholders("SELECT * FROM users"),
            "SELECT * FROM users"
        );
    }

    #[test]
    fn test_convert_to_numbered_placeholders_in_string_literal() {
        assert_eq!(
            convert_to_numbered_placeholders("SELECT * FROM users WHERE name = '?' AND id = ?"),
            "SELECT * FROM users WHERE name = '?' AND id = $1"
        );
    }

    #[test]
    fn test_convert_to_numbered_placeholders_in_quoted_identifier() {
        assert_eq!(
            convert_to_numbered_placeholders(r#"SELECT * FROM "table?" WHERE id = ?"#),
            r#"SELECT * FROM "table?" WHERE id = $1"#
        );
    }

    #[test]
    fn test_convert_to_at_placeholders() {
        assert_eq!(
            convert_to_at_placeholders("SELECT * FROM users WHERE id = ? AND name = ?"),
            "SELECT * FROM users WHERE id = @P1 AND name = @P2"
        );
    }

    #[test]
    fn test_convert_to_at_placeholders_no_params() {
        assert_eq!(
            convert_to_at_placeholders("SELECT * FROM users"),
            "SELECT * FROM users"
        );
    }

    #[test]
    fn test_convert_to_at_placeholders_in_string_literal() {
        assert_eq!(
            convert_to_at_placeholders("SELECT * FROM users WHERE name = '?' AND id = ?"),
            "SELECT * FROM users WHERE name = '?' AND id = @P1"
        );
    }

    #[test]
    fn test_convert_to_at_placeholders_in_quoted_identifier() {
        assert_eq!(
            convert_to_at_placeholders(r#"SELECT * FROM "table?" WHERE id = ?"#),
            r#"SELECT * FROM "table?" WHERE id = @P1"#
        );
    }
}
