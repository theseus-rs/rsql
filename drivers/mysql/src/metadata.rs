use indoc::indoc;
use rsql_driver::{Column, Connection, Index, Metadata, Result, Schema, Table, Value};

pub(crate) async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let mut metadata = Metadata::with_dialect(connection.dialect());

    retrieve_schemas(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_schemas(connection: &mut dyn Connection, metadata: &mut Metadata) -> Result<()> {
    let mut schemas = vec![];
    let sql = indoc! { r"
        SELECT
            schema_name,
            schema_name = database() AS current_schema
        FROM
            information_schema.schemata
        ORDER BY
            schema_name
    "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let schema_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let current = matches!(row.get(1), Some(Value::I16(1)));
        let schema = Schema::new(schema_name, current);
        schemas.push(schema);
    }

    for mut schema in schemas {
        if schema.current() {
            retrieve_tables(connection, &mut schema).await?;
            retrieve_indexes(connection, &mut schema).await?;
        }
        metadata.add(schema);
    }

    Ok(())
}

async fn retrieve_tables(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! { r"
            SELECT
                table_name,
                column_name,
                data_type,
                character_maximum_length,
                is_nullable,
                column_default
            FROM
                information_schema.columns
            WHERE
                table_schema = DATABASE()
            ORDER BY
                table_name,
                ordinal_position
        "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_type = match row.get(2) {
            Some(value) => {
                let character_maximum_length = row.get(3).unwrap_or(&Value::Null);

                if character_maximum_length.is_null() {
                    value.to_string()
                } else {
                    format!("{value}({character_maximum_length})")
                }
            }
            None => continue,
        };
        let not_null = match row.get(4) {
            Some(value) => value.to_string() == "NO",
            None => continue,
        };
        let default_value = match row.get(5) {
            Some(value) => {
                if value.is_null() {
                    None
                } else {
                    Some(value.to_string())
                }
            }
            None => continue,
        };

        let column = Column::new(column_name, column_type, not_null, default_value);
        if let Some(table) = schema.get_mut(&table_name) {
            table.add_column(column);
        } else {
            let mut table = Table::new(table_name);
            table.add_column(column);
            schema.add(table);
        }
    }

    Ok(())
}

async fn retrieve_indexes(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! {r"
            SELECT DISTINCT
                table_name,
                index_name,
                column_name,
                non_unique,
                seq_in_index
            FROM
                information_schema.statistics
            WHERE
                table_schema = DATABASE()
            ORDER BY
                table_name,
                index_name,
                seq_in_index
        "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let index_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(2) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let unique = match row.get(3) {
            Some(value) => value.to_string() == "0",
            _ => continue,
        };
        let Some(table) = schema.get_mut(table_name) else {
            continue;
        };

        if let Some(index) = table.get_index_mut(&index_name) {
            index.add_column(column_name);
        } else {
            let index = Index::new(index_name, vec![column_name.clone()], unique);
            table.add_index(index);
        }
    }

    Ok(())
}
