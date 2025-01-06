use crate::error::Result;
use crate::postgresql::driver::Connection as PgConnection;
use crate::{Metadata, QueryResult};
use async_trait::async_trait;
use file_type::FileType;
use sqlparser::dialect::{Dialect, RedshiftSqlDialect};

#[derive(Debug)]
pub struct Driver;

#[async_trait]
impl crate::Driver for Driver {
    fn identifier(&self) -> &'static str {
        "redshift"
    }

    async fn connect(
        &self,
        url: String,
        password: Option<String>,
    ) -> Result<Box<dyn crate::Connection>> {
        let connection = Connection::new(url, password).await?;
        Ok(Box::new(connection))
    }

    fn supports_file_type(&self, _file_type: &FileType) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Connection {
    url: String,
    inner: PgConnection,
}

impl Connection {
    pub async fn new(url: String, password: Option<String>) -> Result<Self> {
        let inner = PgConnection::new(url.clone(), password).await?;
        Ok(Self { url, inner })
    }
}

#[async_trait]
impl crate::Connection for Connection {
    fn url(&self) -> &String {
        &self.url
    }

    async fn execute(&mut self, sql: &str) -> Result<u64> {
        self.inner.execute(sql).await
    }

    async fn query(&mut self, sql: &str) -> Result<Box<dyn QueryResult>> {
        self.inner.query(sql).await
    }

    async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }

    async fn metadata(&mut self) -> Result<Metadata> {
        self.inner.metadata().await
    }

    fn dialect(&self) -> Box<dyn Dialect> {
        Box::new(RedshiftSqlDialect {})
    }
}
