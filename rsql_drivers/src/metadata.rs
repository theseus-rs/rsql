use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    databases: Vec<Database>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            databases: Vec::new(),
        }
    }

    pub fn add(&mut self, database: Database) {
        self.databases.push(database);
    }

    pub fn get<S: Into<String>>(&self, name: S) -> Option<&Database> {
        let name = name.into();
        self.databases.iter().find(|d| d.name == name)
    }

    pub fn current_database(&self) -> Option<&Database> {
        self.databases.first()
    }

    pub fn databases(&self) -> &[Database] {
        &self.databases
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Database {
    name: String,
    tables: Vec<Table>,
}

impl Database {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            tables: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add(&mut self, table: Table) {
        self.tables.push(table);
    }

    pub fn get<S: Into<String>>(&self, name: S) -> Option<&Table> {
        let name = name.into();
        self.tables.iter().find(|t| t.name == name)
    }

    pub fn get_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Table> {
        let name = name.into();
        self.tables.iter_mut().find(|t| t.name == name)
    }

    pub fn tables(&self) -> &[Table] {
        &self.tables
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Table {
    name: String,
    columns: Vec<Column>,
    indexes: Vec<Index>,
}

impl Table {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            indexes: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn get_column<S: Into<String>>(&self, name: S) -> Option<&Column> {
        let name = name.into();
        self.columns.iter().find(|c| c.name == name)
    }

    pub fn add_index(&mut self, index: Index) {
        self.indexes.push(index);
    }

    pub fn get_index<S: Into<String>>(&self, name: S) -> Option<&Index> {
        let name = name.into();
        self.indexes.iter().find(|i| i.name == name)
    }

    pub fn indexes(&self) -> &[Index] {
        &self.indexes
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Column {
    name: String,
    data_type: String,
    is_nullable: bool,
}

impl Column {
    pub fn new<S: Into<String>>(name: S, data_type: S, is_nullable: bool) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            is_nullable,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn data_type(&self) -> &str {
        &self.data_type
    }

    pub fn is_nullable(&self) -> bool {
        self.is_nullable
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Index {
    name: String,
    columns: Vec<String>,
    is_unique: bool,
}

impl Index {
    pub fn new<S: Into<String>>(name: S, columns: Vec<String>, is_unique: bool) -> Self {
        Self {
            name: name.into(),
            columns,
            is_unique,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    pub fn is_unique(&self) -> bool {
        self.is_unique
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
    }

    #[test]
    fn test_table() {
        let mut table = Table::new("users");
        assert_eq!(table.name(), "users");
        assert_eq!(table.columns().len(), 0);
        assert_eq!(table.indexes().len(), 0);

        let column = Column::new("id", "INTEGER", false);
        table.add_column(column);
        assert_eq!(table.columns().len(), 1);
        assert!(table.get_column("id").is_some());

        let index = Index::new("users_id_idx", vec!["id".to_string()], true);
        table.add_index(index);
        assert_eq!(table.indexes().len(), 1);
        assert!(table.get_index("users_id_idx").is_some());
    }

    #[test]
    fn test_column() {
        let column = Column::new("id", "INTEGER", false);
        assert_eq!(column.name(), "id");
        assert_eq!(column.data_type(), "INTEGER");
        assert_eq!(column.is_nullable(), false);
    }

    #[test]
    fn test_index() {
        let index = Index::new("users_id_idx", vec!["id".to_string()], true);
        assert_eq!(index.name(), "users_id_idx");
        assert_eq!(index.columns(), &["id".to_string()]);
        assert_eq!(index.is_unique(), true);
    }
}
