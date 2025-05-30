use aws_sdk_dynamodb::Client;
use rsql_driver::Error::IoError;
use rsql_driver::{Catalog, Column, Connection, Index, Metadata, Result, Schema, Table};

pub(crate) async fn get_metadata(connection: &dyn Connection, client: &Client) -> Result<Metadata> {
    let mut metadata = Metadata::with_dialect(connection.dialect());

    retrieve_catalogs(client, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_catalogs(client: &Client, metadata: &mut Metadata) -> Result<()> {
    let mut catalogs = vec![Catalog::new("default", true)];
    catalogs.sort_by_key(|catalog| catalog.name().to_ascii_lowercase());

    for mut catalog in catalogs {
        retrieve_schemas(client, &mut catalog).await?;
        metadata.add(catalog);
    }

    Ok(())
}

async fn retrieve_schemas(client: &Client, catalog: &mut Catalog) -> Result<()> {
    let mut schemas = vec![];
    let schema_name = "dynamodb";
    let schema = Schema::new(schema_name, true);
    schemas.push(schema);

    for mut schema in schemas {
        if schema.current() {
            retrieve_tables(client, &mut schema).await?;
        }
        catalog.add(schema);
    }

    Ok(())
}

async fn retrieve_tables(client: &Client, schema: &mut Schema) -> Result<()> {
    let tables = client
        .list_tables()
        .send()
        .await
        .map_err(|error| IoError(format!("{error:?}")))?;
    let table_names = tables.table_names();

    for table_name in table_names {
        let table_description = client
            .describe_table()
            .table_name(table_name)
            .send()
            .await
            .map_err(|error| IoError(format!("{error:?}")))?;

        if let Some(description) = table_description.table() {
            let mut table = Table::new(table_name);

            for attribute in description.attribute_definitions() {
                let name = attribute.attribute_name.to_string();
                let data_type = attribute.attribute_type.to_string();
                let column = Column::new(name, data_type, false, None);
                table.add_column(column);
            }

            for key in description.key_schema() {
                let key_name = key.attribute_name.to_string();
                let key_type = key.key_type.to_string();
                let unique = key_type == "HASH";
                let index = Index::new(key_name.to_string(), vec![key_name], unique);
                table.add_index(index);
            }

            schema.add(table);
        }
    }
    Ok(())
}
