use crate::driver::{Connection as FlightSqlConnection, convert_flight_info_to_query_result};
use arrow_flight::sql::CommandGetDbSchemas;
use rsql_driver::Error::IoError;
use rsql_driver::{Catalog, Connection, Metadata, Result, Schema};
use std::collections::HashMap;

pub(crate) async fn get_metadata(connection: &mut FlightSqlConnection) -> Result<Metadata> {
    let mut metadata = Metadata::with_dialect(connection.dialect());

    retrieve_catalogs(connection, &mut metadata).await?;
    retrieve_schemas(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_catalogs(
    connection: &mut FlightSqlConnection,
    metadata: &mut Metadata,
) -> Result<()> {
    let client = connection.client_mut();
    let flight_info = client
        .get_catalogs()
        .await
        .map_err(|error| IoError(error.to_string()))?;
    let mut query_result = convert_flight_info_to_query_result(client, &flight_info).await?;

    let mut catalogs = Vec::new();
    while let Some(row) = query_result.next().await {
        let catalog_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let catalog = Catalog::new(catalog_name, false);
        catalogs.push(catalog);
    }

    catalogs.sort_by_key(|catalog| catalog.name().to_ascii_lowercase());
    for catalog in catalogs {
        metadata.add(catalog);
    }

    Ok(())
}

async fn retrieve_schemas(
    connection: &mut FlightSqlConnection,
    metadata: &mut Metadata,
) -> Result<()> {
    let client = connection.client_mut();
    let request = CommandGetDbSchemas {
        catalog: None,
        db_schema_filter_pattern: None,
    };
    let flight_info = client
        .get_db_schemas(request)
        .await
        .map_err(|error| IoError(error.to_string()))?;
    let mut query_result = convert_flight_info_to_query_result(client, &flight_info).await?;

    let mut schema_map = HashMap::<String, Vec<Schema>>::new();
    while let Some(row) = query_result.next().await {
        let catalog_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let schema_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };

        let schema = Schema::new(schema_name, false);
        if let Some(schemas) = schema_map.get_mut(&catalog_name) {
            schemas.push(schema);
        } else {
            schema_map.insert(catalog_name, vec![schema]);
        }
    }

    let mut catalogs = schema_map.keys().cloned().collect::<Vec<_>>();
    catalogs.sort_by_key(|name| name.to_ascii_lowercase());

    let first_catalog = catalogs.first().cloned().unwrap_or_default();
    for catalog_name in catalogs {
        let Some(catalog) = metadata.get_mut(catalog_name) else {
            continue;
        };

        if catalog.name() == first_catalog {
            catalog.set_current(true);
        }

        if let Some(mut schemas) = schema_map.remove(catalog.name()) {
            schemas.sort_by_key(|schema| schema.name().to_ascii_lowercase());
            for schema in schemas {
                // TODO: Tables and indexes should be retrieved here
                catalog.add(schema);
            }
        }
    }

    Ok(())
}
