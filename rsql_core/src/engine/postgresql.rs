use crate::engine::QueryResult;
use anyhow::{bail, Result};
use async_trait::async_trait;
use num_format::{Locale, ToFormattedString};
use postgresql_archive::Version;
use postgresql_embedded::{PostgreSQL, Settings};
use rust_decimal::prelude::ToPrimitive;
use sqlx::postgres::{PgColumn, PgConnectOptions, PgRow};
use sqlx::{Column, PgPool, Row};
use std::ops::Deref;
use std::str::FromStr;
use std::string::ToString;

const POSTGRESQL_EMBEDDED_VERSION: &str = "16.2.3";

pub(crate) struct Engine {
    locale: Locale,
    postgresql: Option<PostgreSQL>,
    pool: PgPool,
}

impl Engine {
    pub(crate) async fn new(url: &str) -> Result<Engine> {
        let mut database_url = url.to_string();
        let postgresql = if url.starts_with("postgresql::embedded:") {
            let version = Version::from_str(POSTGRESQL_EMBEDDED_VERSION)?;
            let mut postgresql = PostgreSQL::new(version, Settings::default());
            postgresql.setup().await?;
            postgresql.start().await?;

            let database_name = "embedded";
            postgresql.create_database(database_name).await?;
            let settings = postgresql.settings();
            database_url = settings.url(database_name);
            Some(postgresql)
        } else {
            None
        };

        let options = PgConnectOptions::from_str(database_url.as_str())?;
        let pool = PgPool::connect_with(options).await?;
        let engine = Engine {
            locale: Locale::en,
            postgresql,
            pool,
        };

        Ok(engine)
    }
}

#[async_trait]
impl crate::engine::Engine for Engine {
    async fn execute(&self, sql: &str) -> Result<u64> {
        Ok(sqlx::query(sql).execute(&self.pool).await?.rows_affected())
    }

    async fn query(&self, sql: &str) -> Result<QueryResult> {
        let query_rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let columns = if let Some(row) = query_rows.first() {
            row.columns()
                .iter()
                .map(|column| column.name().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let mut rows = Vec::new();
        for row in query_rows {
            let mut row_data = Vec::new();
            for column in row.columns() {
                let value = self
                    .convert_to_string(&row, column)?
                    .unwrap_or_else(|| "NULL".to_string());
                row_data.push(value);
            }
            rows.push(row_data);
        }

        Ok(QueryResult { columns, rows })
    }

    async fn tables(&mut self) -> Result<Vec<String>> {
        let sql = "SELECT table_name FROM information_schema.tables \
            WHERE table_schema = 'public' ORDER BY table_name";
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let mut tables = Vec::new();

        for row in rows {
            match row.try_get::<String, _>(0) {
                Ok(table_name) => tables.push(table_name),
                Err(error) => bail!("Error: {:?}", error),
            }
        }

        Ok(tables)
    }

    async fn stop(&mut self) -> Result<()> {
        self.pool.close().await;

        if let Some(postgresql) = &self.postgresql {
            match postgresql.stop().await {
                Ok(_) => Ok(()),
                Err(error) => bail!("Error stopping engine: {:?}", error),
            }
        } else {
            Ok(())
        }
    }
}

impl Engine {
    fn convert_to_string(&self, row: &PgRow, column: &PgColumn) -> Result<Option<String>> {
        let column_name = column.name();

        if let Ok(value) = row.try_get(column_name) {
            let value: Option<String> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<&str> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i8> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i16> = value;
            Ok(value.map(|v| v.to_formatted_string(&self.locale)))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i32> = value;
            Ok(value.map(|v| v.to_formatted_string(&self.locale)))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<i64> = value;
            Ok(value.map(|v| v.to_formatted_string(&self.locale)))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<f32> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<f64> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<rust_decimal::Decimal> = value;

            match value {
                Some(v) if v.is_integer() => {
                    Ok(v.to_i128().map(|v| v.to_formatted_string(&self.locale)))
                }
                Some(v) => Ok(Some(v.to_string())),
                None => Ok(None),
            }
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<chrono::NaiveDate> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<chrono::NaiveTime> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<chrono::NaiveDateTime> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<chrono::DateTime<chrono::Utc>> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<serde_json::Value> = value;
            Ok(value.map(|v| v.to_string()))
        } else if let Ok(value) = row.try_get(column_name) {
            let value: Option<uuid::Uuid> = value;
            Ok(value.map(|v| v.to_string()))
        } else {
            let column_type = column.type_info().deref();
            bail!(
                "column type [{:?}] not supported for column [{}]",
                column_type,
                column_name
            );
        }
    }
}
