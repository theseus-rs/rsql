use std::{any::Any, ops::Deref};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlparser::dialect::{self, Dialect};

/// Metadata contains the catalog, schema, table, column, and index definitions for a data source.
///
/// ```text
/// ┌────────────────────────┐
/// │ CATALOG (database)     │
/// │ ┌───────────────────┐  │
/// │ │SCHEMA (namespace) │  │
/// │ │┌────────┬────────┐│  │
/// │ ││ TABLE  │ TABLE  ││  │
/// │ ││ row    │ row    ││  │
/// │ ││ row    │ row    ││  │
/// │ ││ row    │ row    ││  │
/// │ │└────────┴────────┘│  │
/// │ └───────────────────┘  │
/// │ ┌───────────────────┐  │
/// │ │SCHEMA (namespace) │  │
/// │ │┌────────┬────────┐│  │
/// │ ││ TABLE  │ TABLE  ││  │
/// │ ││ row    │ row    ││  │
/// │ ││ row    │ row    ││  │
/// │ ││ row    │ row    ││  │
/// │ │└────────┴────────┘│  │
/// │ └───────────────────┘  │
/// └────────────────────────┘
/// ```
///
/// ```text
/// ┌───────────────────────────────────┐
/// │              TABLE                │
/// ├───────────┬───────────┬───────────┤
/// │ Column A  │ Column B  │ Column C  │
/// ├───────────┼───────────┼───────────┤
/// │ Value A1  │ Value B1  │ Value C1  │ ← Row 1
/// │ Value A2  │ Value B2  │ Value C2  │ ← Row 2
/// │ Value A3  │ Value B3  │ Value C3  │ ← Row 3
/// │     ⋮     │     ⋮     │     ⋮     │
/// │ Value An  │ Value Bn  │ Value Cn  │ ← Row n
/// └───────────┴───────────┴───────────┘
/// ```
///
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    dialect: MetadataDialect,
    catalogs: IndexMap<String, Catalog>,
}

impl Metadata {
    /// Creates a new Metadata instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            dialect: MetadataDialect::Generic,
            catalogs: IndexMap::new(),
        }
    }

    /// Creates a new Metadata instance with the specified dialect.
    #[must_use]
    pub fn with_dialect(dialect: Box<dyn Dialect>) -> Self {
        Self {
            dialect: dialect.into(),
            catalogs: IndexMap::new(),
        }
    }

    /// Returns the dialect for the metadata.
    #[must_use]
    pub fn dialect(&self) -> Box<dyn Dialect> {
        self.dialect.into()
    }

    /// Adds a catalog to the metadata.
    pub fn add(&mut self, catalog: Catalog) {
        self.catalogs.insert(catalog.name.clone(), catalog);
    }

    /// Returns the catalog with the specified name.
    pub fn get<S: Into<String>>(&self, name: S) -> Option<&Catalog> {
        let name = name.into();
        self.catalogs.get(&name)
    }

    /// Returns the mutable catalog with the specified name.
    pub fn get_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Catalog> {
        let name = name.into();
        self.catalogs.get_mut(&name)
    }

    /// Returns the current catalog.
    #[must_use]
    pub fn current_catalog(&self) -> Option<&Catalog> {
        self.catalogs.values().find(|catalog| catalog.current)
    }

    /// Returns the catalogs in the metadata.
    #[must_use]
    pub fn catalogs(&self) -> Vec<&Catalog> {
        let values: Vec<&Catalog> = self.catalogs.values().collect();
        values
    }

    /// Returns the current schema of the current catalog
    #[must_use]
    pub fn current_catalog_schema(&self) -> Option<&Schema> {
        self.current_catalog()
            .and_then(|catalog| catalog.current_schema())
    }
}

/// Catalogs contains the schemas in a data source.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Catalog {
    name: String,
    current: bool,
    schemas: IndexMap<String, Schema>,
}

impl Catalog {
    /// Creates a new Catalog instance.
    pub fn new<S: Into<String>>(name: S, current: bool) -> Self {
        Self {
            name: name.into(),
            current,
            schemas: IndexMap::new(),
        }
    }

    /// Returns the name of the catalog.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the catalog is the current catalog.
    #[must_use]
    pub fn current(&self) -> bool {
        self.current
    }

    /// Sets the current catalog.
    pub fn set_current(&mut self, current: bool) {
        self.current = current;
    }

    /// Adds a schema to the catalog.
    pub fn add(&mut self, schema: Schema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Returns the schema with the specified name.
    pub fn get<S: Into<String>>(&self, name: S) -> Option<&Schema> {
        let name = name.into();
        self.schemas.get(&name)
    }

    /// Returns the mutable schema with the specified name.
    pub fn get_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Schema> {
        let name = name.into();
        self.schemas.get_mut(&name)
    }

    /// Returns the current schema in the catalog.
    #[must_use]
    pub fn current_schema(&self) -> Option<&Schema> {
        self.schemas.values().find(|schema| schema.current)
    }

    /// Returns the schemas in the catalog.
    #[must_use]
    pub fn schemas(&self) -> Vec<&Schema> {
        let values: Vec<&Schema> = self.schemas.values().collect();
        values
    }
}

/// Schema contains the table, column, and index definitions for a schema in a data source.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Schema {
    name: String,
    current: bool,
    tables: IndexMap<String, Table>,
}

impl Schema {
    /// Creates a new Schema instance.
    pub fn new<S: Into<String>>(name: S, current: bool) -> Self {
        Self {
            name: name.into(),
            current,
            tables: IndexMap::new(),
        }
    }

    /// Returns the name of the schema.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the schema is the current schema.
    #[must_use]
    pub fn current(&self) -> bool {
        self.current
    }

    /// Sets the current schema.
    pub fn set_current(&mut self, current: bool) {
        self.current = current;
    }

    /// Adds a table to the schema.
    pub fn add(&mut self, table: Table) {
        self.tables.insert(table.name.clone(), table);
    }

    /// Returns the table with the specified name.
    pub fn get<S: Into<String>>(&self, name: S) -> Option<&Table> {
        let name = name.into();
        self.tables.get(&name)
    }

    /// Returns the mutable table with the specified name.
    pub fn get_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Table> {
        let name = name.into();
        self.tables.get_mut(&name)
    }

    /// Returns the tables in the schema.
    #[must_use]
    pub fn tables(&self) -> Vec<&Table> {
        let values: Vec<&Table> = self.tables.values().collect();
        values
    }
}

/// Table contains the column and index definitions for a table in a schema.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Table {
    name: String,
    columns: IndexMap<String, Column>,
    indexes: IndexMap<String, Index>,
}

impl Table {
    /// Creates a new Table instance.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            columns: IndexMap::new(),
            indexes: IndexMap::new(),
        }
    }

    /// Returns the name of the table.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a column to the table.
    pub fn add_column(&mut self, column: Column) {
        self.columns.insert(column.name.clone(), column);
    }

    /// Returns the column with the specified name.
    #[must_use]
    pub fn columns(&self) -> Vec<&Column> {
        let values: Vec<&Column> = self.columns.values().collect();
        values
    }

    /// Returns the mutable column with the specified name.
    pub fn get_column<S: Into<String>>(&self, name: S) -> Option<&Column> {
        let name = name.into();
        self.columns.get(&name)
    }

    /// Returns the mutable column with the specified name.
    pub fn get_column_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Column> {
        let name = name.into();
        self.columns.get_mut(&name)
    }

    /// Adds an index to the table.
    pub fn add_index(&mut self, index: Index) {
        self.indexes.insert(index.name.clone(), index);
    }

    /// Returns the indexes in the table.
    pub fn get_index<S: Into<String>>(&self, name: S) -> Option<&Index> {
        let name = name.into();
        self.indexes.get(&name)
    }

    /// Returns the mutable index with the specified name.
    pub fn get_index_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Index> {
        let name = name.into();
        self.indexes.get_mut(&name)
    }

    /// Returns the indexes in the table.
    #[must_use]
    pub fn indexes(&self) -> Vec<&Index> {
        let values: Vec<&Index> = self.indexes.values().collect();
        values
    }
}

/// Column contains the definition for a column in a table.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Column {
    name: String,
    data_type: String,
    not_null: bool,
    default: Option<String>,
}

impl Column {
    /// Creates a new Column instance.
    pub fn new<S: Into<String>>(name: S, data_type: S, not_null: bool, default: Option<S>) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            not_null,
            default: default.map(Into::into),
        }
    }

    /// Returns the name of the column.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the data type of the column.
    #[must_use]
    pub fn data_type(&self) -> &str {
        &self.data_type
    }

    /// Returns whether the column is not null.
    #[must_use]
    pub fn not_null(&self) -> bool {
        self.not_null
    }

    /// Returns the default value of the column.
    #[must_use]
    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }
}

/// Index contains the definition for an index on a table.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Index {
    name: String,
    columns: Vec<String>,
    unique: bool,
}

impl Index {
    /// Creates a new Index instance.
    pub fn new<S: Into<String>>(name: S, columns: Vec<S>, unique: bool) -> Self {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(Into::into).collect(),
            unique,
        }
    }

    /// Returns the name of the index.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a column to the index.
    pub fn add_column<S: Into<String>>(&mut self, column: S) {
        self.columns.push(column.into());
    }

    /// The columns in the index.
    #[must_use]
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Returns whether the index is unique.
    #[must_use]
    pub fn unique(&self) -> bool {
        self.unique
    }
}

/// The SQL dialect for the metadata.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum MetadataDialect {
    #[default]
    Generic,
    MySql,
    PostgreSql,
    MsSql,
    Redshift,
    SQLite,
    DuckDb,
    Snowflake,
}

impl From<Box<dyn Dialect>> for MetadataDialect {
    fn from(value: Box<dyn Dialect>) -> Self {
        let types = vec![
            (dialect::GenericDialect.type_id(), Self::Generic),
            (dialect::MySqlDialect {}.type_id(), Self::MySql),
            (dialect::PostgreSqlDialect {}.type_id(), Self::PostgreSql),
            (dialect::MsSqlDialect {}.type_id(), Self::MsSql),
            (dialect::RedshiftSqlDialect {}.type_id(), Self::Redshift),
            (dialect::SQLiteDialect {}.type_id(), Self::SQLite),
            (dialect::DuckDbDialect {}.type_id(), Self::DuckDb),
            (dialect::SnowflakeDialect {}.type_id(), Self::Snowflake),
        ];
        types
            .into_iter()
            .find_map(|(type_id, rsql_dialect)| {
                if value.deref().type_id() == type_id {
                    Some(rsql_dialect)
                } else {
                    None
                }
            })
            .unwrap_or(Self::Generic)
    }
}

impl From<MetadataDialect> for Box<dyn Dialect> {
    fn from(value: MetadataDialect) -> Self {
        match value {
            MetadataDialect::Generic => Box::new(dialect::GenericDialect),
            MetadataDialect::MySql => Box::new(dialect::MySqlDialect {}),
            MetadataDialect::PostgreSql => Box::new(dialect::PostgreSqlDialect {}),
            MetadataDialect::MsSql => Box::new(dialect::MsSqlDialect {}),
            MetadataDialect::Redshift => Box::new(dialect::RedshiftSqlDialect {}),
            MetadataDialect::SQLite => Box::new(dialect::SQLiteDialect {}),
            MetadataDialect::DuckDb => Box::new(dialect::DuckDbDialect {}),
            MetadataDialect::Snowflake => Box::new(dialect::SnowflakeDialect {}),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_metadata() {
        let mut metadata = Metadata::new();
        assert_eq!(
            (*metadata.dialect()).type_id(),
            dialect::GenericDialect {}.type_id()
        );
        assert!(metadata.catalogs().is_empty());
        let catalog = Catalog::new("catalog", true);
        metadata.add(catalog);
        assert!(metadata.get("catalog").is_some());
        assert!(metadata.get_mut("catalog").is_some());
        assert_eq!(metadata.catalogs().len(), 1);
        assert!(metadata.current_catalog().is_some());

        let dialect = Box::new(dialect::PostgreSqlDialect {});
        let metadata = Metadata::with_dialect(dialect);
        assert_eq!(
            (*metadata.dialect()).type_id(),
            dialect::PostgreSqlDialect {}.type_id()
        );
    }

    #[test]
    fn test_catalog() {
        let mut catalog = Catalog::new("catalog", true);
        assert_eq!(catalog.name(), "catalog");
        assert!(catalog.current());
        assert_eq!(catalog.schemas().len(), 0);

        let schema = Schema::new("schema", true);
        catalog.add(schema.clone());
        assert_eq!(catalog.schemas().len(), 1);
        assert!(catalog.get("schema").is_some());
        assert!(catalog.get_mut("schema").is_some());
    }

    #[test]
    fn test_schema() {
        let mut db = Schema::new("default", true);
        assert_eq!(db.name(), "default");
        assert_eq!(db.tables().len(), 0);

        let table = Table::new("users");
        db.add(table.clone());
        assert_eq!(db.tables().len(), 1);
        assert!(db.get("users").is_some());
        assert!(db.get_mut("users").is_some());
    }

    #[test]
    fn test_table() {
        let mut table = Table::new("users");
        assert_eq!(table.name(), "users");
        assert_eq!(table.columns().len(), 0);
        assert_eq!(table.indexes().len(), 0);

        let column = Column::new("id", "INTEGER", false, None);
        table.add_column(column);
        assert_eq!(table.columns().len(), 1);
        assert!(table.get_column("id").is_some());
        assert!(table.get_column_mut("id").is_some());

        let index = Index::new("users_id_idx", vec!["id"], true);
        table.add_index(index);
        assert_eq!(table.indexes().len(), 1);
        assert!(table.get_index("users_id_idx").is_some());
        assert!(table.get_index_mut("users_id_idx").is_some());
    }

    #[test]
    fn test_column() {
        let column = Column::new("id", "INTEGER", false, None);
        assert_eq!(column.name(), "id");
        assert_eq!(column.data_type(), "INTEGER");
        assert!(!column.not_null());
        assert_eq!(column.default(), None);
    }

    #[test]
    fn test_index() {
        let mut index = Index::new("users_id_idx", vec!["id"], true);
        index.add_column("email");
        assert_eq!(index.name(), "users_id_idx");
        assert_eq!(index.columns(), &["id".to_string(), "email".to_string()]);
        assert!(index.unique());
    }

    #[test]
    fn test_current_catalog() {
        let mut metadata = Metadata::new();
        let catalog = Catalog::new("default", true);
        metadata.add(catalog);
        assert_eq!(metadata.current_catalog().unwrap().name(), "default");
    }
}
