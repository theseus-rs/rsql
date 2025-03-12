use crate::error::Result;
use crate::{Metadata, Value};
use async_trait::async_trait;
use mockall::automock;
use mockall::predicate::str;
use sqlparser::ast::Statement;
use sqlparser::dialect::{Dialect, GenericDialect};
use sqlparser::parser::Parser;

use jiff::civil::DateTime;
use jiff::tz::Offset;
use jiff::{Timestamp, ToSpan};
use std::fmt::Debug;

/// A single row of a query result
pub type Row = Vec<Value>;

/// Results from a query
#[async_trait]
pub trait QueryResult: Debug + Send + Sync {
    async fn columns(&self) -> Vec<String>;
    async fn next(&mut self) -> Option<Row>;
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
    async fn columns(&self) -> Vec<String> {
        self.inner.columns().await
    }

    async fn next(&mut self) -> Option<Vec<Value>> {
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
    async fn columns(&self) -> Vec<String> {
        self.columns.clone()
    }

    async fn next(&mut self) -> Option<Vec<Value>> {
        let result = self.rows.get(self.row_index).cloned();
        self.row_index += 1;
        result
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
#[automock]
#[async_trait]
pub trait Connection: Debug + Send + Sync {
    fn url(&self) -> &String;
    async fn execute(&mut self, sql: &str) -> Result<u64>;
    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>>;
    async fn close(&mut self) -> Result<()>;
    async fn metadata(&mut self) -> Result<Metadata> {
        unimplemented!()
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
    metadata: Option<Metadata>,
    timestamp: DateTime,
}

impl CachedMetadataConnection {
    #[must_use]
    pub fn new(connection: Box<dyn Connection>) -> Self {
        let now = Offset::UTC.to_datetime(Timestamp::now());
        Self {
            connection,
            metadata: None,
            timestamp: now,
        }
    }
}

#[async_trait]
impl Connection for CachedMetadataConnection {
    fn url(&self) -> &String {
        self.connection.url()
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        if let StatementMetadata::DDL = self.parse_sql(sql) {
            self.metadata = None;
        }

        self.connection.execute(sql).await
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
            let metadata = self.connection.metadata().await?;
            self.metadata = Some(metadata.clone());
            Ok(metadata)
        }
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        self.connection.query(sql).await
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::Value;

    #[tokio::test]
    async fn test_memory_query_result_new() {
        let columns = vec!["a".to_string()];
        let rows = vec![vec![Value::String("foo".to_string())]];

        let mut result = MemoryQueryResult::new(columns, rows);

        let columns = result.columns().await;
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

        let columns = result.columns().await;
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

        let columns = result.columns().await;
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

        async fn execute(&mut self, _sql: &str) -> Result<u64> {
            Ok(0)
        }

        async fn metadata(&mut self) -> Result<Metadata> {
            Ok(Metadata::default())
        }

        async fn query(&mut self, _sql: &str) -> Result<Box<dyn QueryResult>> {
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
}
