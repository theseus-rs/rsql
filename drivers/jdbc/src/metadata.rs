use crate::Connection;
use crate::connection::{invoke, java_string, jdbc_error};
use crate::results::read_result;
use ristretto_vm::{VM, Value as JavaValue};
use rsql_driver::{
    Catalog, Column, ForeignKey, Index, Metadata, PrimaryKey, QueryResult, Result, Schema, Table,
    Value, View,
};
use std::collections::BTreeMap;

type Record = BTreeMap<String, Value>;

fn escape_pattern(name: &str, escape: &str) -> String {
    if escape.is_empty() {
        return name.to_owned();
    }
    name.replace(escape, &format!("{escape}{escape}"))
        .replace('%', &format!("{escape}%"))
        .replace('_', &format!("{escape}_"))
}

async fn records(
    vm: &VM,
    metadata: &JavaValue,
    method: &str,
    arguments: &[JavaValue],
) -> Result<Vec<Record>> {
    let result_set = invoke(vm, metadata, method, arguments).await?;
    let result = read_result(vm, &result_set).await;
    let close = invoke(vm, &result_set, "close()V", &[]).await;
    let mut result = result?;
    close?;
    let columns = result.columns().to_vec();
    let mut rows = Vec::new();
    while let Some(row) = result.next().await {
        rows.push(columns.iter().cloned().zip(row.iter().cloned()).collect());
    }
    Ok(rows)
}

fn string(record: &Record, name: &str) -> String {
    record
        .get(name)
        .filter(|value| !value.is_null())
        .map(ToString::to_string)
        .unwrap_or_default()
}

fn optional(record: &Record, name: &str) -> Option<String> {
    record
        .get(name)
        .filter(|value| !value.is_null())
        .map(ToString::to_string)
}

fn ordinal(record: &Record, name: &str) -> i32 {
    string(record, name).parse().unwrap_or_default()
}

pub(crate) async fn metadata(connection: &Connection) -> Result<Metadata> {
    let vm = &connection.vm;
    let jdbc = invoke(
        vm,
        &connection.inner,
        "getMetaData()Ljava/sql/DatabaseMetaData;",
        &[],
    )
    .await?;
    let current_catalog =
        invoke(vm, &connection.inner, "getCatalog()Ljava/lang/String;", &[]).await?;
    let current_schema =
        invoke(vm, &connection.inner, "getSchema()Ljava/lang/String;", &[]).await?;
    let catalog_name = if current_catalog.is_null() {
        String::new()
    } else {
        current_catalog.as_string().map_err(jdbc_error)?
    };
    let schema_name = if current_schema.is_null() {
        String::new()
    } else {
        current_schema.as_string().map_err(jdbc_error)?
    };
    let mut metadata = Metadata::new();
    for record in records(vm, &jdbc, "getCatalogs()Ljava/sql/ResultSet;", &[]).await? {
        let name = string(&record, "TABLE_CAT");
        metadata.add(Catalog::new(name.clone(), name == catalog_name));
    }
    if metadata.get(&catalog_name).is_none() {
        metadata.add(Catalog::new(catalog_name.clone(), true));
    }
    for record in records(vm, &jdbc, "getSchemas()Ljava/sql/ResultSet;", &[]).await? {
        let name = string(&record, "TABLE_SCHEM");
        let catalog = optional(&record, "TABLE_CATALOG").unwrap_or_else(|| catalog_name.clone());
        if metadata.get(&catalog).is_none() {
            metadata.add(Catalog::new(catalog.clone(), catalog == catalog_name));
        }
        if let Some(catalog) = metadata.get_mut(&catalog) {
            catalog.add(Schema::new(name.clone(), name == schema_name));
        }
    }
    let catalog = metadata
        .get_mut(&catalog_name)
        .ok_or_else(|| jdbc_error("Missing current catalog"))?;
    if catalog.get(&schema_name).is_none() {
        catalog.add(Schema::new(schema_name.clone(), true));
    }
    let schema = catalog
        .get_mut(&schema_name)
        .ok_or_else(|| jdbc_error("Missing current schema"))?;
    tables(vm, &jdbc, &current_catalog, &current_schema, schema).await?;
    Ok(metadata)
}

async fn tables(
    vm: &VM,
    jdbc: &JavaValue,
    current_catalog: &JavaValue,
    current_schema: &JavaValue,
    schema: &mut Schema,
) -> Result<()> {
    let escape = invoke(vm, jdbc, "getSearchStringEscape()Ljava/lang/String;", &[])
        .await?
        .as_string()
        .map_err(jdbc_error)?;
    let schema_pattern = if current_schema.is_null() {
        current_schema.clone()
    } else {
        java_string(
            vm,
            &escape_pattern(&current_schema.as_string().map_err(jdbc_error)?, &escape),
        )
        .await?
    };
    let wildcard = java_string(vm, "%").await?;
    let tables = records(vm, jdbc, "getTables(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;)Ljava/sql/ResultSet;", &[current_catalog.clone(), schema_pattern.clone(), wildcard, JavaValue::Object(None)]).await?;
    for record in tables {
        let name = string(&record, "TABLE_NAME");
        let kind = string(&record, "TABLE_TYPE");
        let arguments = [
            current_catalog.clone(),
            current_schema.clone(),
            java_string(vm, &name).await?,
        ];
        let column_arguments = [
            current_catalog.clone(),
            schema_pattern.clone(),
            java_string(vm, &escape_pattern(&name, &escape)).await?,
            JavaValue::Object(None),
        ];
        let mut columns = records(vm, jdbc, "getColumns(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/sql/ResultSet;", &column_arguments).await?;
        columns.sort_by_key(|record| ordinal(record, "ORDINAL_POSITION"));
        let columns: Vec<Column> = columns
            .iter()
            .map(|record| {
                Column::new(
                    string(record, "COLUMN_NAME"),
                    string(record, "TYPE_NAME"),
                    string(record, "IS_NULLABLE") == "NO",
                    optional(record, "COLUMN_DEF"),
                )
            })
            .collect();
        if kind.contains("VIEW") {
            let mut view = View::new(name);
            for column in columns {
                view.add_column(column);
            }
            schema.add_view(view);
        } else {
            let mut table = Table::new(name);
            for column in columns {
                table.add_column(column);
            }
            keys(vm, jdbc, &arguments, &mut table).await?;
            schema.add(table);
        }
    }
    Ok(())
}

async fn keys(vm: &VM, jdbc: &JavaValue, arguments: &[JavaValue], table: &mut Table) -> Result<()> {
    let mut primary = records(vm, jdbc, "getPrimaryKeys(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/sql/ResultSet;", arguments).await?;
    primary.sort_by_key(|record| ordinal(record, "KEY_SEQ"));
    if let Some(first) = primary.first() {
        table.set_primary_key(PrimaryKey::new(
            string(first, "PK_NAME"),
            primary
                .iter()
                .map(|row| string(row, "COLUMN_NAME"))
                .collect(),
            false,
        ));
    }
    let mut index_arguments = arguments.to_vec();
    index_arguments.extend([JavaValue::from(false), JavaValue::from(true)]);
    let indexes = records(vm, jdbc, "getIndexInfo(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;ZZ)Ljava/sql/ResultSet;", &index_arguments).await?;
    let mut groups: BTreeMap<String, Vec<Record>> = BTreeMap::new();
    for row in indexes {
        if let Some(name) = optional(&row, "INDEX_NAME") {
            groups.entry(name).or_default().push(row);
        }
    }
    for (name, mut rows) in groups {
        rows.sort_by_key(|row| ordinal(row, "ORDINAL_POSITION"));
        let unique = rows
            .first()
            .is_some_and(|row| matches!(row.get("NON_UNIQUE"), Some(Value::Bool(false))));
        table.add_index(Index::new(
            name,
            rows.iter()
                .filter_map(|row| optional(row, "COLUMN_NAME"))
                .collect(),
            unique,
        ));
    }
    let foreign = records(vm, jdbc, "getImportedKeys(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/sql/ResultSet;", arguments).await?;
    let mut groups: BTreeMap<String, Vec<Record>> = BTreeMap::new();
    for row in foreign {
        groups.entry(string(&row, "FK_NAME")).or_default().push(row);
    }
    for (name, mut rows) in groups {
        rows.sort_by_key(|row| ordinal(row, "KEY_SEQ"));
        if let Some(first) = rows.first() {
            table.add_foreign_key(ForeignKey::new(
                name,
                rows.iter()
                    .map(|row| string(row, "FKCOLUMN_NAME"))
                    .collect(),
                string(first, "PKTABLE_NAME"),
                rows.iter()
                    .map(|row| string(row, "PKCOLUMN_NAME"))
                    .collect(),
                false,
            ));
        }
    }
    Ok(())
}
