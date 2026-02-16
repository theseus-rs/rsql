use crate::Connection as PolarsConnection;
use polars::datatypes::{DataType, Field};
use polars::prelude::SchemaExt;
use rsql_driver::Error::IoError;
use rsql_driver::{Catalog, Column, Metadata, Result, Schema, Table};

pub(crate) async fn get_metadata(connection: &mut PolarsConnection) -> Result<Metadata> {
    let mut metadata = Metadata::default();

    retrieve_catalogs(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_catalogs(
    connection: &mut PolarsConnection,
    metadata: &mut Metadata,
) -> Result<()> {
    let mut catalogs = vec![Catalog::new("default", true)];
    catalogs.sort_by_key(|catalog| catalog.name().to_ascii_lowercase());

    for mut catalog in catalogs {
        retrieve_schemas(connection, &mut catalog).await?;
        metadata.add(catalog);
    }

    Ok(())
}

async fn retrieve_schemas(connection: &mut PolarsConnection, catalog: &mut Catalog) -> Result<()> {
    let mut schemas = vec![Schema::new("polars", true)];

    schemas.sort_by_key(|schema| schema.name().to_ascii_lowercase());

    for mut schema in schemas {
        if schema.current() {
            retrieve_tables(connection, &mut schema).await?;
        }
        catalog.add(schema);
    }

    Ok(())
}

async fn retrieve_tables(connection: &mut PolarsConnection, schema: &mut Schema) -> Result<()> {
    let context = connection.context();
    let mut context = context.lock().await;
    let table_names = context.get_tables();

    for table_name in table_names {
        let lazy_frame = context
            .execute(&format!("SELECT * FROM \"{table_name}\" LIMIT 0"))
            .map_err(|error| IoError(error.to_string()))?;
        let data_frame = lazy_frame
            .collect()
            .map_err(|error: polars::prelude::PolarsError| IoError(error.to_string()))?;
        let data_frame_schema = data_frame.schema();

        let mut table = Table::new(&table_name);
        for field in data_frame_schema.iter_fields() {
            add_table_column(&mut table, &String::new(), &field);
        }
        schema.add(table);
    }

    Ok(())
}

fn add_table_column(table: &mut Table, column_prefix: &String, field: &Field) {
    let column_name = if column_prefix.is_empty() {
        field.name().to_string()
    } else {
        format!("{column_prefix}.{}", field.name())
    };
    let data_type = field.dtype();
    let column_type = data_type.to_string();
    let column = Column::new(column_name.clone(), column_type, false, None);
    table.add_column(column);

    match data_type {
        DataType::List(inner_data_type) => {
            if let DataType::Struct(fields) = inner_data_type.as_ref() {
                let column_prefix = format!("{column_name}[]");
                for field in fields {
                    add_table_column(table, &column_prefix, field);
                }
            }
        }
        DataType::Struct(fields) => {
            for field in fields {
                add_table_column(table, &column_name, field);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use polars::prelude::*;
    use polars_sql::SQLContext;
    use rsql_driver::Connection;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_metadata() -> Result<()> {
        let ids = Series::new("id".into(), &[1i64, 2i64]);
        let names = Series::new("name".into(), &["John Doe", "Jane Smith"]);
        let data_frame = DataFrame::new_infer_height(vec![
            polars::prelude::Column::from(ids),
            polars::prelude::Column::from(names),
        ])
        .map_err(|error| IoError(error.to_string()))?;
        let context = SQLContext::new();
        context.register("users", data_frame.lazy());
        let mut connection = PolarsConnection::new("polars://", context).await?;

        let metadata = connection.metadata().await?;
        assert_eq!(metadata.catalogs().len(), 1);
        let catalog = metadata.current_catalog().expect("catalog");
        assert_eq!(catalog.schemas().len(), 1);
        let schema = catalog.current_schema().expect("schema");
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
