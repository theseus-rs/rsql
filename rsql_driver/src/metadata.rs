use i18n_inflector::{LanguageRuleSet, LanguageRules};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlparser::dialect::{self, Dialect};
use std::{any::Any, ops::Deref};

/// Metadata contains the catalog, schema, table, column, index, primary key, and foreign key
/// definitions for a data source.
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

    /// Infers primary keys for all schemas in all catalogs.
    pub fn infer_primary_keys(&mut self, language_rules: &LanguageRuleSet) {
        for catalog in self.catalogs.values_mut() {
            for schema in catalog.schemas.values_mut() {
                schema.infer_primary_keys(language_rules);
            }
        }
    }

    /// Infers foreign keys for all schemas in all catalogs.
    pub fn infer_foreign_keys(&mut self, language_rules: &LanguageRuleSet) {
        for catalog in self.catalogs.values_mut() {
            for schema in catalog.schemas.values_mut() {
                schema.infer_foreign_keys(language_rules);
            }
        }
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

/// Schema contains the table, view, column, index, primary key, and foreign key definitions for a
/// schema in a data source.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Schema {
    name: String,
    current: bool,
    tables: IndexMap<String, Table>,
    views: IndexMap<String, View>,
}

impl Schema {
    /// Creates a new Schema instance.
    pub fn new<S: Into<String>>(name: S, current: bool) -> Self {
        Self {
            name: name.into(),
            current,
            tables: IndexMap::new(),
            views: IndexMap::new(),
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

    /// Adds a view to the schema.
    pub fn add_view(&mut self, view: View) {
        self.views.insert(view.name.clone(), view);
    }

    /// Returns the view with the specified name.
    pub fn get_view<S: Into<String>>(&self, name: S) -> Option<&View> {
        let name = name.into();
        self.views.get(&name)
    }

    /// Returns the mutable view with the specified name.
    pub fn get_view_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut View> {
        let name = name.into();
        self.views.get_mut(&name)
    }

    /// Returns the views in the schema.
    #[must_use]
    pub fn views(&self) -> Vec<&View> {
        let values: Vec<&View> = self.views.values().collect();
        values
    }

    /// Infers primary keys for tables that don't have one declared.
    ///
    /// For each table without a primary key, this method looks for:
    /// 1. A NOT NULL column named `id`
    /// 2. A NOT NULL column named `<table_name>_id` (e.g., `user_id` for table `users`)
    ///
    /// If found, an inferred primary key is created with `inferred: true`.
    pub fn infer_primary_keys(&mut self, language_rules: &LanguageRuleSet) {
        let table_names: Vec<String> = self.tables.keys().cloned().collect();

        for table_name in &table_names {
            if let Some(table) = self.tables.get(table_name) {
                if table.primary_key.is_some() {
                    continue;
                }

                // First, try to find a NOT NULL column named "id"
                if let Some(column) = table.find_column_case_insensitive("id")
                    && column.not_null()
                {
                    let pk_name = format!("inferred_{table_name}_pk");
                    let pk = PrimaryKey::new(pk_name, vec![column.name().to_string()], true);
                    if let Some(table) = self.tables.get_mut(table_name) {
                        table.set_primary_key(pk);
                    }
                    continue;
                }

                // Second, try to find a NOT NULL column named "<singular_table_name>_id"
                let singular_name = language_rules.singularize(table_name);
                let pk_column_name = format!("{singular_name}_id");

                if let Some(column) = table.find_column_case_insensitive(&pk_column_name)
                    && column.not_null()
                {
                    let pk_name = format!("inferred_{table_name}_pk");
                    let pk = PrimaryKey::new(pk_name, vec![column.name().to_string()], true);
                    if let Some(table) = self.tables.get_mut(table_name) {
                        table.set_primary_key(pk);
                    }
                }
            }
        }
    }

    /// Infers foreign keys for all tables in the schema based on column naming conventions.
    ///
    /// For each column matching the pattern `<name>_id`, this method looks for a table named
    /// `<name>` or a plural form of `<name>` in the same schema. If a match is found and the
    /// referenced table has a column named `id`, an inferred foreign key is created. Columns
    /// that already have a declared foreign key are skipped.
    pub fn infer_foreign_keys(&mut self, language_rules: &LanguageRuleSet) {
        let table_names: Vec<String> = self.tables.keys().cloned().collect();

        for table_name in &table_names {
            let mut inferred_fks: Vec<ForeignKey> = Vec::new();

            if let Some(table) = self.tables.get(table_name) {
                let existing_fk_columns: Vec<String> = table
                    .foreign_keys
                    .values()
                    .flat_map(|fk| fk.columns.clone())
                    .collect();

                for column in table.columns.values() {
                    let column_name = column.name();
                    if !column_name.to_ascii_lowercase().ends_with("_id") {
                        continue;
                    }
                    if existing_fk_columns
                        .iter()
                        .any(|c| c.eq_ignore_ascii_case(column_name))
                    {
                        continue;
                    }

                    let col_name_lower = column_name.to_ascii_lowercase();
                    let prefix = &col_name_lower[..col_name_lower.len() - 3];
                    let candidates = language_rules.pluralize(prefix);

                    for candidate in &candidates {
                        if candidate.eq_ignore_ascii_case(table_name) {
                            continue;
                        }
                        let ref_table = self
                            .tables
                            .iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case(candidate.as_ref()))
                            .map(|(_, v)| v);
                        if let Some(ref_table) = ref_table
                            && ref_table.find_column_case_insensitive("id").is_some()
                        {
                            let fk_name = format!("inferred_{table_name}_{column_name}_fk");
                            inferred_fks.push(ForeignKey::new(
                                fk_name,
                                vec![column_name.to_string()],
                                ref_table.name().to_string(),
                                vec!["id".to_string()],
                                true,
                            ));
                            break;
                        }
                    }
                }
            }

            if let Some(table) = self.tables.get_mut(table_name) {
                for fk in inferred_fks {
                    table.add_foreign_key(fk);
                }
            }
        }
    }
}

/// Table contains the column, index, primary key, and foreign key definitions for a table
/// in a schema.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Table {
    name: String,
    columns: IndexMap<String, Column>,
    indexes: IndexMap<String, Index>,
    primary_key: Option<PrimaryKey>,
    foreign_keys: IndexMap<String, ForeignKey>,
}

impl Table {
    /// Creates a new Table instance.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            columns: IndexMap::new(),
            indexes: IndexMap::new(),
            primary_key: None,
            foreign_keys: IndexMap::new(),
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

    /// Returns the column with the specified name.
    pub fn get_column<S: Into<String>>(&self, name: S) -> Option<&Column> {
        let name = name.into();
        self.columns.get(&name)
    }

    /// Returns the column matching the specified name using a case-insensitive comparison.
    fn find_column_case_insensitive(&self, name: &str) -> Option<&Column> {
        self.columns
            .values()
            .find(|column| column.name.eq_ignore_ascii_case(name))
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

    /// Sets the primary key for the table.
    pub fn set_primary_key(&mut self, primary_key: PrimaryKey) {
        self.primary_key = Some(primary_key);
    }

    /// Returns the primary key of the table.
    #[must_use]
    pub fn primary_key(&self) -> Option<&PrimaryKey> {
        self.primary_key.as_ref()
    }

    /// Adds a foreign key to the table.
    pub fn add_foreign_key(&mut self, foreign_key: ForeignKey) {
        self.foreign_keys
            .insert(foreign_key.name.clone(), foreign_key);
    }

    /// Returns the foreign key with the specified name.
    pub fn get_foreign_key<S: Into<String>>(&self, name: S) -> Option<&ForeignKey> {
        let name = name.into();
        self.foreign_keys.get(&name)
    }

    /// Returns the mutable foreign key with the specified name.
    pub fn get_foreign_key_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut ForeignKey> {
        let name = name.into();
        self.foreign_keys.get_mut(&name)
    }

    /// Returns the foreign keys in the table.
    #[must_use]
    pub fn foreign_keys(&self) -> Vec<&ForeignKey> {
        let values: Vec<&ForeignKey> = self.foreign_keys.values().collect();
        values
    }
}

/// View contains the column definitions for a view in a schema.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct View {
    name: String,
    columns: IndexMap<String, Column>,
}

impl View {
    /// Creates a new View instance.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            columns: IndexMap::new(),
        }
    }

    /// Returns the name of the view.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a column to the view.
    pub fn add_column(&mut self, column: Column) {
        self.columns.insert(column.name.clone(), column);
    }

    /// Returns the columns of the view.
    #[must_use]
    pub fn columns(&self) -> Vec<&Column> {
        let values: Vec<&Column> = self.columns.values().collect();
        values
    }

    /// Returns the column with the specified name.
    pub fn get_column<S: Into<String>>(&self, name: S) -> Option<&Column> {
        let name = name.into();
        self.columns.get(&name)
    }

    /// Returns the mutable column with the specified name.
    pub fn get_column_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Column> {
        let name = name.into();
        self.columns.get_mut(&name)
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

/// PrimaryKey contains the definition for a primary key constraint on a table.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PrimaryKey {
    name: String,
    columns: Vec<String>,
    inferred: bool,
}

impl PrimaryKey {
    /// Creates a new PrimaryKey instance.
    pub fn new<S: Into<String>>(name: S, columns: Vec<S>, inferred: bool) -> Self {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(Into::into).collect(),
            inferred,
        }
    }

    /// Returns the name of the primary key.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the columns in the primary key.
    #[must_use]
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Returns whether the primary key was inferred rather than declared.
    #[must_use]
    pub fn inferred(&self) -> bool {
        self.inferred
    }
}

/// ForeignKey contains the definition for a foreign key constraint on a table.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ForeignKey {
    name: String,
    columns: Vec<String>,
    referenced_table: String,
    referenced_columns: Vec<String>,
    inferred: bool,
}

impl ForeignKey {
    /// Creates a new ForeignKey instance.
    pub fn new<S: Into<String>>(
        name: S,
        columns: Vec<S>,
        referenced_table: S,
        referenced_columns: Vec<S>,
        inferred: bool,
    ) -> Self {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(Into::into).collect(),
            referenced_table: referenced_table.into(),
            referenced_columns: referenced_columns.into_iter().map(Into::into).collect(),
            inferred,
        }
    }

    /// Returns the name of the foreign key.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the local columns in the foreign key.
    #[must_use]
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Returns the referenced table name.
    #[must_use]
    pub fn referenced_table(&self) -> &str {
        &self.referenced_table
    }

    /// Returns the referenced columns in the foreign key.
    #[must_use]
    pub fn referenced_columns(&self) -> &[String] {
        &self.referenced_columns
    }

    /// Returns whether the foreign key was inferred rather than declared.
    #[must_use]
    pub fn inferred(&self) -> bool {
        self.inferred
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
    use i18n_inflector::language_rules;

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
        assert_eq!(db.views().len(), 0);

        let table = Table::new("users");
        db.add(table.clone());
        assert_eq!(db.tables().len(), 1);
        assert!(db.get("users").is_some());
        assert!(db.get_mut("users").is_some());

        let view = View::new("active_users");
        db.add_view(view);
        assert_eq!(db.views().len(), 1);
        assert!(db.get_view("active_users").is_some());
        assert!(db.get_view_mut("active_users").is_some());
    }

    #[test]
    fn test_view() {
        let mut view = View::new("active_users");
        assert_eq!(view.name(), "active_users");
        assert_eq!(view.columns().len(), 0);

        let column = Column::new("id", "INTEGER", false, None);
        view.add_column(column);
        assert_eq!(view.columns().len(), 1);
        assert!(view.get_column("id").is_some());
        assert!(view.get_column_mut("id").is_some());
    }

    #[test]
    fn test_table() {
        let mut table = Table::new("users");
        assert_eq!(table.name(), "users");
        assert_eq!(table.columns().len(), 0);
        assert_eq!(table.indexes().len(), 0);
        assert!(table.primary_key().is_none());
        assert_eq!(table.foreign_keys().len(), 0);

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

        let pk = PrimaryKey::new("users_pkey", vec!["id"], false);
        table.set_primary_key(pk);
        assert!(table.primary_key().is_some());
        let pk = table.primary_key().unwrap();
        assert_eq!(pk.name(), "users_pkey");
        assert_eq!(pk.columns(), &["id".to_string()]);
        assert!(!pk.inferred());

        let fk = ForeignKey::new(
            "fk_users_org",
            vec!["org_id"],
            "organizations",
            vec!["id"],
            false,
        );
        table.add_foreign_key(fk);
        assert_eq!(table.foreign_keys().len(), 1);
        assert!(table.get_foreign_key("fk_users_org").is_some());
        assert!(table.get_foreign_key_mut("fk_users_org").is_some());
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

    #[test]
    fn test_foreign_key() {
        let fk = ForeignKey::new(
            "fk_orders_user",
            vec!["user_id"],
            "users",
            vec!["id"],
            false,
        );
        assert_eq!(fk.name(), "fk_orders_user");
        assert_eq!(fk.columns(), &["user_id".to_string()]);
        assert_eq!(fk.referenced_table(), "users");
        assert_eq!(fk.referenced_columns(), &["id".to_string()]);
        assert!(!fk.inferred());
    }

    #[test]
    fn test_foreign_key_inferred() {
        let fk = ForeignKey::new(
            "inferred_orders_user_id_fk",
            vec!["user_id"],
            "users",
            vec!["id"],
            true,
        );
        assert!(fk.inferred());
    }

    #[test]
    fn test_infer_foreign_keys() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("id", "INTEGER", true, None));
        users.add_column(Column::new("name", "TEXT", false, None));
        schema.add(users);

        let mut orders = Table::new("orders");
        orders.add_column(Column::new("id", "INTEGER", true, None));
        orders.add_column(Column::new("user_id", "INTEGER", false, None));
        schema.add(orders);

        let language_rules = language_rules("en").unwrap();
        schema.infer_foreign_keys(language_rules);

        let orders = schema.get("orders").unwrap();
        assert_eq!(orders.foreign_keys().len(), 1);
        let fk = orders
            .get_foreign_key("inferred_orders_user_id_fk")
            .unwrap();
        assert_eq!(fk.columns(), &["user_id".to_string()]);
        assert_eq!(fk.referenced_table(), "users");
        assert_eq!(fk.referenced_columns(), &["id".to_string()]);
        assert!(fk.inferred());
    }

    #[test]
    fn test_infer_foreign_keys_plural_table() {
        let mut schema = Schema::new("public", true);

        let mut categories = Table::new("categories");
        categories.add_column(Column::new("id", "INTEGER", true, None));
        schema.add(categories);

        let mut products = Table::new("products");
        products.add_column(Column::new("id", "INTEGER", true, None));
        products.add_column(Column::new("category_id", "INTEGER", false, None));
        schema.add(products);

        let language_rules = language_rules("en").unwrap();
        schema.infer_foreign_keys(language_rules);

        let products = schema.get("products").unwrap();
        assert_eq!(products.foreign_keys().len(), 1);
        let fk = products
            .get_foreign_key("inferred_products_category_id_fk")
            .unwrap();
        assert_eq!(fk.referenced_table(), "categories");
    }

    #[test]
    fn test_infer_foreign_keys_skips_existing() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("id", "INTEGER", true, None));
        schema.add(users);

        let mut orders = Table::new("orders");
        orders.add_column(Column::new("id", "INTEGER", true, None));
        orders.add_column(Column::new("user_id", "INTEGER", false, None));
        orders.add_foreign_key(ForeignKey::new(
            "fk_orders_user",
            vec!["user_id"],
            "users",
            vec!["id"],
            false,
        ));
        schema.add(orders);

        let language_rules = language_rules("en").unwrap();
        schema.infer_foreign_keys(language_rules);

        let orders = schema.get("orders").unwrap();
        assert_eq!(orders.foreign_keys().len(), 1);
        assert!(orders.get_foreign_key("fk_orders_user").is_some());
    }

    #[test]
    fn test_infer_foreign_keys_no_id_column() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("user_id", "INTEGER", true, None));
        schema.add(users);

        let mut orders = Table::new("orders");
        orders.add_column(Column::new("user_id", "INTEGER", false, None));
        schema.add(orders);

        let language_rules = language_rules("en").unwrap();
        schema.infer_foreign_keys(language_rules);

        let orders = schema.get("orders").unwrap();
        assert_eq!(orders.foreign_keys().len(), 0);
    }

    #[test]
    fn test_infer_foreign_keys_no_matching_table() {
        let mut schema = Schema::new("public", true);

        let mut orders = Table::new("orders");
        orders.add_column(Column::new("id", "INTEGER", true, None));
        orders.add_column(Column::new("customer_id", "INTEGER", false, None));
        schema.add(orders);

        let language_rules = language_rules("en").unwrap();
        schema.infer_foreign_keys(language_rules);

        let orders = schema.get("orders").unwrap();
        assert_eq!(orders.foreign_keys().len(), 0);
    }

    #[test]
    fn test_primary_key() {
        let pk = PrimaryKey::new("users_pkey", vec!["id"], false);
        assert_eq!(pk.name(), "users_pkey");
        assert_eq!(pk.columns(), &["id".to_string()]);
        assert!(!pk.inferred());
    }

    #[test]
    fn test_primary_key_inferred() {
        let pk = PrimaryKey::new("inferred_users_pk", vec!["id"], true);
        assert!(pk.inferred());
    }

    #[test]
    fn test_primary_key_composite() {
        let pk = PrimaryKey::new("orders_pkey", vec!["order_id", "line_id"], false);
        assert_eq!(
            pk.columns(),
            &["order_id".to_string(), "line_id".to_string()]
        );
    }

    #[test]
    fn test_infer_primary_keys() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("id", "INTEGER", true, None));
        users.add_column(Column::new("name", "TEXT", false, None));
        schema.add(users);

        let language_rules = language_rules("en").unwrap();
        schema.infer_primary_keys(language_rules);

        let users = schema.get("users").unwrap();
        let pk = users.primary_key().unwrap();
        assert_eq!(pk.name(), "inferred_users_pk");
        assert_eq!(pk.columns(), &["id".to_string()]);
        assert!(pk.inferred());
    }

    #[test]
    fn test_infer_primary_keys_skips_existing() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("id", "INTEGER", true, None));
        users.set_primary_key(PrimaryKey::new("users_pkey", vec!["id"], false));
        schema.add(users);

        let language_rules = language_rules("en").unwrap();
        schema.infer_primary_keys(language_rules);

        let users = schema.get("users").unwrap();
        let pk = users.primary_key().unwrap();
        assert_eq!(pk.name(), "users_pkey");
        assert!(!pk.inferred());
    }

    #[test]
    fn test_infer_primary_keys_table_name_id_column() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("user_id", "INTEGER", true, None));
        schema.add(users);

        let language_rules = language_rules("en").unwrap();
        schema.infer_primary_keys(language_rules);

        let users = schema.get("users").unwrap();
        let pk = users.primary_key().unwrap();
        assert_eq!(pk.name(), "inferred_users_pk");
        assert_eq!(pk.columns(), &["user_id".to_string()]);
        assert!(pk.inferred());
    }

    #[test]
    fn test_infer_primary_keys_singular_table_name() {
        let mut schema = Schema::new("public", true);

        let mut user = Table::new("user");
        user.add_column(Column::new("user_id", "INTEGER", true, None));
        schema.add(user);

        let language_rules = language_rules("en").unwrap();
        schema.infer_primary_keys(language_rules);

        let user = schema.get("user").unwrap();
        let pk = user.primary_key().unwrap();
        assert_eq!(pk.name(), "inferred_user_pk");
        assert_eq!(pk.columns(), &["user_id".to_string()]);
        assert!(pk.inferred());
    }

    #[test]
    fn test_infer_primary_keys_no_matching_column() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("other_id", "INTEGER", true, None));
        schema.add(users);

        let language_rules = language_rules("en").unwrap();
        schema.infer_primary_keys(language_rules);

        let users = schema.get("users").unwrap();
        assert!(users.primary_key().is_none());
    }

    #[test]
    fn test_infer_primary_keys_nullable_id() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("id", "INTEGER", false, None));
        schema.add(users);

        let language_rules = language_rules("en").unwrap();
        schema.infer_primary_keys(language_rules);

        let users = schema.get("users").unwrap();
        assert!(users.primary_key().is_none());
    }

    #[test]
    fn test_infer_primary_keys_case_insensitive_id() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("ID", "INTEGER", true, None));
        users.add_column(Column::new("name", "TEXT", false, None));
        schema.add(users);

        let language_rules = language_rules("en").unwrap();
        schema.infer_primary_keys(language_rules);

        let users = schema.get("users").unwrap();
        let pk = users.primary_key().unwrap();
        assert_eq!(pk.name(), "inferred_users_pk");
        assert_eq!(pk.columns(), &["ID".to_string()]);
        assert!(pk.inferred());
    }

    #[test]
    fn test_infer_primary_keys_case_insensitive_table_name_id() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("User_Id", "INTEGER", true, None));
        schema.add(users);

        let language_rules = language_rules("en").unwrap();
        schema.infer_primary_keys(language_rules);

        let users = schema.get("users").unwrap();
        let pk = users.primary_key().unwrap();
        assert_eq!(pk.name(), "inferred_users_pk");
        assert_eq!(pk.columns(), &["User_Id".to_string()]);
        assert!(pk.inferred());
    }

    #[test]
    fn test_infer_foreign_keys_case_insensitive_column() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("ID", "INTEGER", true, None));
        users.add_column(Column::new("name", "TEXT", false, None));
        schema.add(users);

        let mut orders = Table::new("orders");
        orders.add_column(Column::new("id", "INTEGER", true, None));
        orders.add_column(Column::new("User_ID", "INTEGER", false, None));
        schema.add(orders);

        let language_rules = language_rules("en").unwrap();
        schema.infer_foreign_keys(language_rules);

        let orders = schema.get("orders").unwrap();
        assert_eq!(orders.foreign_keys().len(), 1);
        let fk = orders
            .get_foreign_key("inferred_orders_User_ID_fk")
            .unwrap();
        assert_eq!(fk.columns(), &["User_ID".to_string()]);
        assert_eq!(fk.referenced_table(), "users");
        assert_eq!(fk.referenced_columns(), &["id".to_string()]);
        assert!(fk.inferred());
    }

    #[test]
    fn test_infer_foreign_keys_case_insensitive_table() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("Users");
        users.add_column(Column::new("id", "INTEGER", true, None));
        schema.add(users);

        let mut orders = Table::new("orders");
        orders.add_column(Column::new("id", "INTEGER", true, None));
        orders.add_column(Column::new("user_id", "INTEGER", false, None));
        schema.add(orders);

        let language_rules = language_rules("en").unwrap();
        schema.infer_foreign_keys(language_rules);

        let orders = schema.get("orders").unwrap();
        assert_eq!(orders.foreign_keys().len(), 1);
        let fk = orders
            .get_foreign_key("inferred_orders_user_id_fk")
            .unwrap();
        assert_eq!(fk.columns(), &["user_id".to_string()]);
        assert_eq!(fk.referenced_table(), "Users");
        assert_eq!(fk.referenced_columns(), &["id".to_string()]);
        assert!(fk.inferred());
    }

    #[test]
    fn test_infer_foreign_keys_case_insensitive_skips_existing() {
        let mut schema = Schema::new("public", true);

        let mut users = Table::new("users");
        users.add_column(Column::new("id", "INTEGER", true, None));
        schema.add(users);

        let mut orders = Table::new("orders");
        orders.add_column(Column::new("id", "INTEGER", true, None));
        orders.add_column(Column::new("USER_ID", "INTEGER", false, None));
        orders.add_foreign_key(ForeignKey::new(
            "fk_orders_user",
            vec!["USER_ID"],
            "users",
            vec!["id"],
            false,
        ));
        schema.add(orders);

        let language_rules = language_rules("en").unwrap();
        schema.infer_foreign_keys(language_rules);

        let orders = schema.get("orders").unwrap();
        assert_eq!(orders.foreign_keys().len(), 1);
        assert!(orders.get_foreign_key("fk_orders_user").is_some());
    }
}
