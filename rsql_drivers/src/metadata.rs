use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    databases: IndexMap<String, Database>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            databases: IndexMap::new(),
        }
    }

    pub fn add(&mut self, database: Database) {
        self.databases.insert(database.name.clone(), database);
    }

    pub fn get<S: Into<String>>(&self, name: S) -> Option<&Database> {
        let name = name.into();
        self.databases.get(&name)
    }

    pub fn get_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Database> {
        let name = name.into();
        self.databases.get_mut(&name)
    }

    pub fn current_database(&self) -> Option<&Database> {
        if let Some((_name, database)) = self.databases.first() {
            Some(database)
        } else {
            None
        }
    }

    pub fn databases(&self) -> Vec<&Database> {
        let values: Vec<&Database> = self.databases.values().collect();
        values
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Database {
    name: String,
    tables: IndexMap<String, Table>,
}

impl Database {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            tables: IndexMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.insert(column.name.clone(), column);
    }

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
            default: default.map(|value| value.into()),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> &str {
        &self.data_type
    }

    pub fn not_null(&self) -> bool {
        self.not_null
    }

    pub fn default(&self) -> Option<&str> {
        self.default.as_deref()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Index {
    name: String,
    columns: Vec<String>,
    primary_key: bool,
    unique: bool,
}

impl Index {
    pub fn new<S: Into<String>>(name: S, columns: Vec<S>, primary_key: bool, unique: bool) -> Self {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(|column| column.into()).collect(),
            primary_key,
            unique,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_column<S: Into<String>>(&mut self, column: S) {
        self.columns.push(column.into());
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    pub fn primary_key(&self) -> bool {
        self.primary_key
    }

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
        assert_eq!(metadata.databases().len(), 0);

        let database = Database::new("default");
        metadata.add(database.clone());
        assert_eq!(metadata.databases().len(), 1);
        assert!(metadata.get("default").is_some());
        assert!(metadata.get_mut("default").is_some());
    }

    #[test]
    fn test_database() {
        let mut db = Database::new("default");
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

        let index = Index::new("users_id_idx", vec!["id"], true, true);
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
        let mut index = Index::new("users_id_idx", vec!["id"], true, true);
        index.add_column("email");
        assert_eq!(index.name(), "users_id_idx");
        assert_eq!(index.columns(), &["id".to_string(), "email".to_string()]);
        assert!(index.primary_key());
        assert!(index.unique());
    }
}
