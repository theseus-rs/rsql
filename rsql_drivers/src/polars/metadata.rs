use crate::polars::Connection as PolarsConnection;
use crate::{Column, Metadata, Result, Schema, Table};

pub(crate) async fn get_metadata(connection: &mut PolarsConnection) -> Result<Metadata> {
    let mut metadata = Metadata::default();

    retrieve_schemas(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_schemas(
    connection: &mut PolarsConnection,
    metadata: &mut Metadata,
) -> Result<()> {
    let mut schemas = vec![Schema::new("polars", true)];

    schemas.sort_by_key(|schema| schema.name().to_string());

    for mut schema in schemas {
        if schema.current() {
            retrieve_tables(connection, &mut schema).await?;
        }
        metadata.add(schema);
    }

    Ok(())
}

async fn retrieve_tables(connection: &mut PolarsConnection, schema: &mut Schema) -> Result<()> {
    let context = connection.context();
    let mut context = context.lock().await;
    let tables = context.get_tables();

    for table_name in tables {
        let sql = format!("SELECT * FROM {table_name} LIMIT 0");
        let result = context.execute(sql.as_str())?;
        let data_frame = result.collect()?;

        let mut table = Table::new(table_name);
        for column in data_frame.get_columns() {
            let column_name = column.name().to_string();
            let column_type = column.dtype().to_string();
            let column = Column::new(column_name, column_type, false, None);
            table.add_column(column);
        }
        schema.add(table);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Connection;
    use polars::prelude::*;
    use polars_sql::SQLContext;

    #[tokio::test]
    async fn test_metadata() -> anyhow::Result<()> {
        let ids = Series::new("id".into(), &[1i64, 2i64]);
        let names = Series::new("name".into(), &["John Doe", "Jane Smith"]);
        let data_frame = DataFrame::new(vec![
            polars::prelude::Column::from(ids),
            polars::prelude::Column::from(names),
        ])?;
        let mut context = SQLContext::new();
        context.register("users", data_frame.lazy());
        let mut connection = PolarsConnection::new("polars://".to_string(), context).await?;

        let metadata = connection.metadata().await?;
        let schema = metadata.current_schema().expect("schema");
        assert_eq!(schema.tables().len(), 1);

        let users_table = schema.get("users").expect("users table");
        assert_eq!(users_table.name(), "users");
        assert_eq!(users_table.columns().len(), 2);
        let id_column = users_table.get_column("id").expect("id column");
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "i64");
        assert!(!id_column.not_null());
        assert_eq!(id_column.default(), None);
        let name_column = users_table.get_column("name").expect("name column");
        assert_eq!(name_column.name(), "name");
        assert_eq!(name_column.data_type(), "str");
        assert!(!name_column.not_null());
        assert_eq!(name_column.default(), None);

        assert!(users_table.indexes().is_empty());

        connection.close().await?;
        Ok(())
    }
}
