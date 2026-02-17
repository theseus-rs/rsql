use indoc::indoc;
use rsql_driver::{
    Catalog, Column, Connection, ForeignKey, Index, Metadata, PrimaryKey, Result, Schema, Table,
    Value, View,
};

pub(crate) async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let mut metadata = Metadata::with_dialect(connection.dialect());

    retrieve_catalogs(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_catalogs(connection: &mut dyn Connection, metadata: &mut Metadata) -> Result<()> {
    let mut catalogs = vec![];
    let sql = indoc! { r"
        SELECT
            name,
            CASE
                WHEN name = DB_NAME() THEN CAST(1 AS BIT)
                ELSE CAST(0 AS BIT)
            END AS current_database
        FROM
            sys.databases;
    "};
    let mut query_result = connection.query(sql, &[]).await?;

    while let Some(row) = query_result.next().await {
        let catalog_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let current = matches!(row.get(1), Some(Value::Bool(true)));
        let catalog = Catalog::new(catalog_name, current);
        catalogs.push(catalog);
    }

    for mut catalog in catalogs {
        retrieve_schemas(connection, &mut catalog).await?;
        metadata.add(catalog);
    }

    Ok(())
}

async fn retrieve_schemas(connection: &mut dyn Connection, catalog: &mut Catalog) -> Result<()> {
    let mut schemas = vec![];
    let sql = indoc! { r"
        SELECT
            name,
            CASE
                WHEN name = SCHEMA_NAME() THEN CAST(1 AS BIT)
                ELSE CAST(0 AS BIT)
            END AS current_schema
        FROM
            sys.schemas;
    "};
    let mut query_result = connection.query(sql, &[]).await?;

    while let Some(row) = query_result.next().await {
        let schema_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let current = matches!(row.get(1), Some(Value::Bool(true)));
        let schema = Schema::new(schema_name, current);
        schemas.push(schema);
    }

    for mut schema in schemas {
        if schema.current() {
            retrieve_tables(connection, &mut schema).await?;
            retrieve_views(connection, &mut schema).await?;
            retrieve_indexes(connection, &mut schema).await?;
            retrieve_primary_keys(connection, &mut schema).await?;
            retrieve_foreign_keys(connection, &mut schema).await?;
        }
        catalog.add(schema);
    }

    Ok(())
}

async fn retrieve_views(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! { r"
            SELECT
                c.TABLE_NAME,
                c.COLUMN_NAME,
                c.DATA_TYPE,
                c.CHARACTER_MAXIMUM_LENGTH,
                c.IS_NULLABLE,
                c.COLUMN_DEFAULT
            FROM
                INFORMATION_SCHEMA.COLUMNS c
                JOIN INFORMATION_SCHEMA.VIEWS v
                    ON c.TABLE_CATALOG = v.TABLE_CATALOG
                    AND c.TABLE_SCHEMA = v.TABLE_SCHEMA
                    AND c.TABLE_NAME = v.TABLE_NAME
            ORDER BY
                c.TABLE_NAME,
                c.ORDINAL_POSITION
        "};
    let mut query_result = connection.query(sql, &[]).await?;

    while let Some(row) = query_result.next().await {
        let view_name = match row.first() {
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
        if let Some(view) = schema.get_view_mut(&view_name) {
            view.add_column(column);
        } else {
            let mut view = View::new(view_name);
            view.add_column(column);
            schema.add_view(view);
        }
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
            ORDER BY
                table_name,
                ordinal_position
        "};
    let mut query_result = connection.query(sql, &[]).await?;

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
            SELECT
                t.name AS table_name,
                i.name AS index_name,
                c.name AS column_name,
                i.is_unique
            FROM
                sys.tables t
                JOIN sys.indexes i ON i.object_id = t.object_id
                JOIN sys.index_columns ic ON ic.object_id = i.object_id AND ic.index_id = i.index_id
                JOIN sys.columns c ON ic.object_id = c.object_id and ic.column_id = c.column_id
            ORDER BY
                t.name,
                i.name,
                ic.key_ordinal
        "};
    let mut query_result = connection.query(sql, &[]).await?;

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
            Some(Value::Bool(value)) => *value,
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

async fn retrieve_primary_keys(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! {r"
            SELECT
                t.name AS table_name,
                kc.name AS constraint_name,
                c.name AS column_name
            FROM
                sys.key_constraints kc
                JOIN sys.tables t ON kc.parent_object_id = t.object_id
                JOIN sys.index_columns ic ON kc.parent_object_id = ic.object_id AND kc.unique_index_id = ic.index_id
                JOIN sys.columns c ON ic.object_id = c.object_id AND ic.column_id = c.column_id
            WHERE
                kc.type = 'PK'
            ORDER BY
                t.name,
                kc.name,
                ic.key_ordinal
        "};
    let mut query_result = connection.query(sql, &[]).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let constraint_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(2) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let Some(table) = schema.get_mut(table_name) else {
            continue;
        };

        if table.primary_key().is_some() {
            continue;
        }

        let pk = PrimaryKey::new(constraint_name, vec![column_name], false);
        table.set_primary_key(pk);
    }

    Ok(())
}

async fn retrieve_foreign_keys(connection: &mut dyn Connection, schema: &mut Schema) -> Result<()> {
    let sql = indoc! {r"
            SELECT
                tp.name AS table_name,
                fk.name AS constraint_name,
                cp.name AS column_name,
                tr.name AS referenced_table_name,
                cr.name AS referenced_column_name
            FROM
                sys.foreign_keys fk
                JOIN sys.tables tp ON fk.parent_object_id = tp.object_id
                JOIN sys.tables tr ON fk.referenced_object_id = tr.object_id
                JOIN sys.foreign_key_columns fkc ON fkc.constraint_object_id = fk.object_id
                JOIN sys.columns cp ON fkc.parent_column_id = cp.column_id AND fkc.parent_object_id = cp.object_id
                JOIN sys.columns cr ON fkc.referenced_column_id = cr.column_id AND fkc.referenced_object_id = cr.object_id
            ORDER BY
                tp.name,
                fk.name,
                fkc.constraint_column_id
        "};
    let mut query_result = connection.query(sql, &[]).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.first() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let constraint_name = match row.get(1) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let column_name = match row.get(2) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let referenced_table = match row.get(3) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let referenced_column = match row.get(4) {
            Some(value) => value.to_string(),
            None => continue,
        };
        let Some(table) = schema.get_mut(table_name) else {
            continue;
        };

        if table.get_foreign_key(&constraint_name).is_some() {
            continue;
        }

        let fk = ForeignKey::new(
            constraint_name,
            vec![column_name],
            referenced_table,
            vec![referenced_column],
            false,
        );
        table.add_foreign_key(fk);
    }

    Ok(())
}
