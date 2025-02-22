use crate::Connection;
use crate::error::Result;
use async_trait::async_trait;
use file_type::FileType;
use mockall::automock;
use mockall::predicate::str;
use std::fmt::Debug;

#[automock]
#[async_trait]
pub trait Driver: Debug + Send + Sync {
    fn identifier(&self) -> &'static str;
    async fn connect(&self, url: &str) -> Result<Box<dyn Connection>>;
    fn supports_file_type(&self, file_type: &FileType) -> bool;
}
