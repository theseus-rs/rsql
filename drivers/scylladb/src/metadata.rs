use crate::Connection;
use crate::convert::type_name;
use rsql_driver::{
    Catalog, Column, Index, Metadata, PrimaryKey, Result, Schema, Table, Value, View,
};
use scylla::cluster::metadata::{ColumnKind, MaterializedView};

const SECONDARY_INDEX_QUERY: &str =
    "SELECT keyspace_name, table_name, index_name, options FROM system_schema.indexes";

pub(crate) async fn get_metadata(connection: &mut Connection) -> Result<Metadata> {
    let current_keyspace = connection
        .session
        .get_session()
        .get_keyspace()
        .map(|keyspace| keyspace.to_string());
    let state = connection.session.get_session().get_cluster_state();
    let catalog_name = if state.cluster_name().is_empty() {
        "scylladb".to_string()
    } else {
        state.cluster_name().to_string()
    };

    let mut catalog = Catalog::new(&catalog_name, true);
    let mut keyspaces = state.keyspaces_iter().collect::<Vec<_>>();
    keyspaces.sort_unstable_by(|left, right| left.0.cmp(right.0));
    for (keyspace_name, keyspace) in keyspaces {
        let mut schema = Schema::new(
            keyspace_name,
            current_keyspace.as_deref() == Some(keyspace_name),
        );

        let mut tables = keyspace.tables.iter().collect::<Vec<_>>();
        tables.sort_unstable_by(|left, right| left.0.cmp(right.0));
        for (table_name, table_metadata) in tables {
            schema.add(table(table_name, table_metadata));
        }

        let mut views = keyspace.views.iter().collect::<Vec<_>>();
        views.sort_unstable_by(|left, right| left.0.cmp(right.0));
        for (view_name, view_metadata) in views {
            schema.add_view(view(view_name, view_metadata));
        }
        catalog.add(schema);
    }
    drop(state);

    let mut metadata = Metadata::new();
    metadata.add(catalog);
    add_secondary_indexes(connection, &mut metadata, &catalog_name).await?;
    Ok(metadata)
}

fn table(name: &str, source: &scylla::cluster::metadata::Table) -> Table {
    let mut table = Table::new(name);
    let mut columns = source.columns.iter().collect::<Vec<_>>();
    columns.sort_unstable_by(|left, right| left.0.cmp(right.0));
    for (column_name, column) in columns {
        let not_null = matches!(
            column.kind,
            ColumnKind::PartitionKey | ColumnKind::Clustering
        );
        table.add_column(Column::new(
            column_name.clone(),
            type_name(&column.typ),
            not_null,
            None::<String>,
        ));
    }

    let primary_columns = source
        .partition_key
        .iter()
        .chain(&source.clustering_key)
        .cloned()
        .collect::<Vec<_>>();
    if !primary_columns.is_empty() {
        table.set_primary_key(PrimaryKey::new(
            "PRIMARY".to_string(),
            primary_columns.clone(),
            false,
        ));
        table.add_index(Index::new("PRIMARY".to_string(), primary_columns, true));
    }
    table
}

fn view(name: &str, source: &MaterializedView) -> View {
    let mut view = View::new(name);
    let mut columns = source.view_metadata.columns.iter().collect::<Vec<_>>();
    columns.sort_unstable_by(|left, right| left.0.cmp(right.0));
    for (column_name, column) in columns {
        let not_null = matches!(
            column.kind,
            ColumnKind::PartitionKey | ColumnKind::Clustering
        );
        view.add_column(Column::new(
            column_name.clone(),
            type_name(&column.typ),
            not_null,
            None::<String>,
        ));
    }
    view
}

async fn add_secondary_indexes(
    connection: &mut Connection,
    metadata: &mut Metadata,
    catalog_name: &str,
) -> Result<()> {
    let mut query_result =
        rsql_driver::Connection::query(connection, SECONDARY_INDEX_QUERY, &[]).await?;
    let mut indexes = Vec::new();
    while let Some(row) = query_result.next().await {
        let (
            Some(Value::String(keyspace)),
            Some(Value::String(table)),
            Some(Value::String(name)),
            Some(Value::Map(options)),
        ) = (row.first(), row.get(1), row.get(2), row.get(3))
        else {
            continue;
        };
        let target_key = Value::String("target".to_string());
        let Some(Value::String(target)) = options.get(&target_key) else {
            continue;
        };
        indexes.push((
            keyspace.clone(),
            table.clone(),
            name.clone(),
            index_column(target),
        ));
    }
    indexes.sort_unstable();

    let Some(catalog) = metadata.get_mut(catalog_name) else {
        return Ok(());
    };
    for (keyspace, table_name, index_name, column) in indexes {
        if let Some(table) = catalog
            .get_mut(keyspace)
            .and_then(|schema| schema.get_mut(table_name))
        {
            table.add_index(Index::new(index_name, vec![column], false));
        }
    }
    Ok(())
}

fn index_column(target: &str) -> String {
    let target = target.trim();
    let inner = ["keys", "values", "entries", "full"]
        .iter()
        .find_map(|prefix| {
            target
                .strip_prefix(prefix)
                .and_then(|value| value.strip_prefix('('))
                .and_then(|value| value.strip_suffix(')'))
        })
        .unwrap_or(target)
        .trim();
    inner
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(inner)
        .replace("\"\"", "\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_index_columns() {
        assert_eq!(index_column("email"), "email");
        assert_eq!(index_column("values(tags)"), "tags");
        assert_eq!(index_column("full(\"Mixed Name\")"), "Mixed Name");
    }
}
