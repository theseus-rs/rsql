use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    schemas: IndexMap<String, Schema>,
}

impl Metadata {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schemas: IndexMap::new(),
        }
    }

    pub fn add(&mut self, schema: Schema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    pub fn get<S: Into<String>>(&self, name: S) -> Option<&Schema> {
        let name = name.into();
        self.schemas.get(&name)
    }

    pub fn get_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Schema> {
        let name = name.into();
        self.schemas.get_mut(&name)
    }

    #[must_use]
    pub fn current_schema(&self) -> Option<&Schema> {
        self.schemas.values().find(|schema| schema.current)
    }

    #[must_use]
    pub fn schemas(&self) -> Vec<&Schema> {
        let values: Vec<&Schema> = self.schemas.values().collect();
        values
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Schema {
    name: String,
    current: bool,
    tables: IndexMap<String, Table>,
}

impl Schema {
    pub fn new<S: Into<String>>(name: S, current: bool) -> Self {
        Self {
            name: name.into(),
            current,
            tables: IndexMap::new(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn current(&self) -> bool {
        self.current
    }

    pub fn add(&mut self, table: Table) {
        self.tables.insert(table.name.clone(), table);
    }

    pub fn get<S: Into<String>>(&self, name: S) -> Option<&Table> {
        let name = name.into();
        self.tables.get(&name)
    }

    pub fn get_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Table> {
        let name = name.into();
        self.tables.get_mut(&name)
    }

    #[must_use]
    pub fn tables(&self) -> Vec<&Table> {
        let values: Vec<&Table> = self.tables.values().collect();
        values
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Table {
    name: String,
    columns: IndexMap<String, Column>,
    indexes: IndexMap<String, Index>,
}

impl Table {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            columns: IndexMap::new(),
            indexes: IndexMap::new(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.insert(column.name.clone(), column);
    }

    #[must_use]
    pub fn columns(&self) -> Vec<&Column> {
        let values: Vec<&Column> = self.columns.values().collect();
        values
    }

    pub fn get_column<S: Into<String>>(&self, name: S) -> Option<&Column> {
        let name = name.into();
        self.columns.get(&name)
    }

    pub fn get_column_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Column> {
        let name = name.into();
        self.columns.get_mut(&name)
    }

    pub fn add_index(&mut self, index: Index) {
        self.indexes.insert(index.name.clone(), index);
    }

    pub fn get_index<S: Into<String>>(&self, name: S) -> Option<&Index> {
        let name = name.into();
        self.indexes.get(&name)
    }

    pub fn get_index_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Index> {
        let name = name.into();
        self.indexes.get_mut(&name)
    }

    #[must_use]
    pub fn indexes(&self) -> Vec<&Index> {
        let values: Vec<&Index> = self.indexes.values().collect();
        values
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Column {
    name: String,
    data_type: String,
    not_null: bool,
    default: Option<String>,
}

impl Column {
    pub fn new<S: Into<String>>(name: S, data_type: S, not_null: bool, default: Option<S>) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            not_null,
            default: default.map(Into::into),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn data_type(&self) -> &str {
        &self.data_type
    }

    #[must_use]
    pub fn not_null(&self) -> bool {
        self.not_null
    }

    #[must_use]
    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Index {
    name: String,
    columns: Vec<String>,
    unique: bool,
}

impl Index {
    pub fn new<S: Into<String>>(name: S, columns: Vec<S>, unique: bool) -> Self {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(Into::into).collect(),
            unique,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_column<S: Into<String>>(&mut self, column: S) {
        self.columns.push(column.into());
    }

    #[must_use]
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    #[must_use]
    pub fn unique(&self) -> bool {
        self.unique
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_metadata() {
        let mut metadata = Metadata::new();
        assert_eq!(metadata.schemas().len(), 0);

        let default_schema = Schema::new("default", true);
        let test_schema = Schema::new("test", false);

        assert!(metadata.current_schema().is_none());
        metadata.add(default_schema.clone());
        metadata.add(test_schema.clone());
        assert_eq!(metadata.schemas().len(), 2);

        let current_schema = metadata.current_schema();
        assert!(current_schema.is_some());
        if let Some(schema) = current_schema {
            assert_eq!(schema.name(), "default");
            assert!(schema.current());
        }
        assert!(metadata.get("default").is_some());
        assert!(metadata.get("default").is_some());
        assert!(metadata.get_mut("default").is_some());
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
}
