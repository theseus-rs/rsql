use rsql_driver::{Catalog, Column, Connection, Index, Metadata, Result, Schema, Table, Value};

pub async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let catalogs = get_catalogs(connection).await?;
    let mut metadata = Metadata::new();
    for catalog in catalogs {
        metadata.add(catalog);
    }
    Ok(metadata)
}

async fn get_catalogs(connection: &mut dyn Connection) -> Result<Vec<Catalog>> {
    let query = "SELECT name FROM system.databases ORDER BY name";
    let mut result = connection.query(query).await?;
    let mut catalogs = Vec::new();

    while let Some(row) = result.next().await {
        if let Some(Value::String(name)) = row.first() {
            let schemas = get_schemas(connection, name).await?;
            let mut catalog = Catalog::new(name, name == "default");
            for schema in schemas {
                catalog.add(schema);
            }
            catalogs.push(catalog);
        }
    }

    Ok(catalogs)
}

async fn get_schemas(connection: &mut dyn Connection, database_name: &str) -> Result<Vec<Schema>> {
    let tables = get_tables(connection, database_name).await?;
    let mut schema = Schema::new("default", true);
    for table in tables {
        schema.add(table);
    }
    Ok(vec![schema])
}

async fn get_tables(connection: &mut dyn Connection, database_name: &str) -> Result<Vec<Table>> {
    let query = format!(
        "SELECT name, engine FROM system.tables WHERE database = '{}' ORDER BY name",
        database_name.replace("'", "''")
    );

    let mut result = connection.query(&query).await?;
    let mut tables = Vec::new();

    while let Some(row) = result.next().await {
        if let (Some(Value::String(name)), Some(Value::String(_engine))) = (row.first(), row.get(1))
        {
            let columns = get_columns(connection, database_name, name).await?;
            let indexes = get_indexes(connection, database_name, name).await?;
            let mut table = Table::new(name);
            for column in columns {
                table.add_column(column);
            }
            for index in indexes {
                table.add_index(index);
            }
            tables.push(table);
        }
    }

    Ok(tables)
}

async fn get_columns(
    connection: &mut dyn Connection,
    database_name: &str,
    table_name: &str,
) -> Result<Vec<Column>> {
    let query = format!(
        "SELECT name, type, default_kind, default_expression, is_in_primary_key
         FROM system.columns
         WHERE database = '{}' AND table = '{}'
         ORDER BY position",
        database_name.replace("'", "''"),
        table_name.replace("'", "''")
    );

    let mut result = connection.query(&query).await?;
    let mut columns = Vec::new();

    while let Some(row) = result.next().await {
        if let Some(Value::String(name)) = row.first() {
            let data_type = match row.get(1) {
                Some(Value::String(type_string)) => type_string.to_string(),
                _ => "String".to_string(),
            };
            let is_nullable = data_type.starts_with("Nullable(");
            let default_value = match row.get(3) {
                Some(Value::String(default_expr)) if !default_expr.is_empty() => Some(default_expr),
                _ => None,
            };

            columns.push(Column::new(name, &data_type, !is_nullable, default_value));
        }
    }

    Ok(columns)
}

async fn get_indexes(
    connection: &mut dyn Connection,
    database_name: &str,
    table_name: &str,
) -> Result<Vec<Index>> {
    let mut indexes = Vec::new();
    if let Ok(primary_key) = get_primary_key(connection, database_name, table_name).await
        && !primary_key.columns().is_empty()
    {
        indexes.push(primary_key);
    }
    let data_indexes = get_data_skipping_indexes(connection, database_name, table_name).await?;
    indexes.extend(data_indexes);
    Ok(indexes)
}

async fn get_primary_key(
    connection: &mut dyn Connection,
    database_name: &str,
    table_name: &str,
) -> Result<Index> {
    let query = format!(
        "SELECT primary_key FROM system.tables
         WHERE database = '{}' AND name = '{}'",
        database_name.replace("'", "''"),
        table_name.replace("'", "''")
    );

    let mut result = connection.query(&query).await?;
    let primary_key_expr = if let Some(row) = result.next().await {
        match row.first() {
            Some(Value::String(expr)) => expr.clone(),
            _ => String::new(),
        }
    } else {
        String::new()
    };

    let columns: Vec<&str> = primary_key_expr
        .split(',')
        .map(|column| column.trim())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(Index::new("PRIMARY", columns, true))
}

async fn get_data_skipping_indexes(
    connection: &mut dyn Connection,
    database_name: &str,
    table_name: &str,
) -> Result<Vec<Index>> {
    let query = format!(
        "SELECT name, expr, type FROM system.data_skipping_indices
         WHERE database = '{}' AND table = '{}'",
        database_name.replace("'", "''"),
        table_name.replace("'", "''")
    );
    let mut result = connection.query(&query).await?;
    let mut indexes = Vec::new();

    while let Some(row) = result.next().await {
        if let (
            Some(Value::String(index_name)),
            Some(Value::String(expr)),
            Some(Value::String(_index_type)),
        ) = (row.first(), row.get(1), row.get(2))
        {
            let columns = vec![expr];
            indexes.push(Index::new(index_name, columns, false));
        }
    }

    Ok(indexes)
}
