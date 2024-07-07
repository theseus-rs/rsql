use crate::{Column, Connection, Index, Metadata, Result, Schema, Table, Value};
use indoc::indoc;

pub(crate) async fn get_metadata(connection: &mut dyn Connection) -> Result<Metadata> {
    let mut metadata = Metadata::default();

    retrieve_schemas(connection, &mut metadata).await?;

    Ok(metadata)
}

async fn retrieve_schemas(connection: &mut dyn Connection, metadata: &mut Metadata) -> Result<()> {
    let mut schemas = vec![];
    let sql = indoc! { r"
        SELECT
            s.name,
            CASE
                WHEN s.name = SCHEMA_NAME() THEN CAST(1 AS BIT)
                ELSE CAST(0 AS BIT)
            END AS current_schema
        FROM
            sys.schemas s;
    "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let schema_name = match row.get(0) {
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
            ORDER BY
                table_name,
                ordinal_position
        "};
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.get(0) {
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
    let mut query_result = connection.query(sql).await?;

    while let Some(row) = query_result.next().await {
        let table_name = match row.get(0) {
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

#[cfg(target_os = "linux")]
#[cfg(test)]
mod test {
    use crate::{Connection, DriverManager};
    use testcontainers::runners::AsyncRunner;
    use testcontainers::ContainerRequest;
    use testcontainers_modules::mssql_server::MssqlServer;

    const PASSWORD: &str = "Password42!";

    #[tokio::test]
    async fn test_container() -> anyhow::Result<()> {
        let sqlserver_image =
            ContainerRequest::from(MssqlServer::default().with_sa_password(PASSWORD));
        let container = sqlserver_image.start().await?;
        let port = container.get_host_port_ipv4(1433).await?;
        let database_url =
            &format!("sqlserver://sa:{PASSWORD}@127.0.0.1:{port}?TrustServerCertificate=true");
        let driver_manager = DriverManager::default();
        let mut connection = driver_manager.connect(database_url.as_str()).await?;

        test_schema(&mut *connection).await?;

        Ok(())
    }

    async fn test_schema(connection: &mut dyn Connection) -> anyhow::Result<()> {
        let _ = connection
            .execute("CREATE TABLE contacts (id INT PRIMARY KEY, email VARCHAR(20))")
            .await?;
        let _ = connection
            .execute("CREATE TABLE users (id INT PRIMARY KEY, email VARCHAR(20))")
            .await?;

        let metadata = connection.metadata().await?;
        let schema = metadata.current_schema().expect("schema");

        let contacts_table = schema.get("contacts").expect("contacts table");
        assert_eq!(contacts_table.name(), "contacts");
        assert_eq!(contacts_table.columns().len(), 2);
        let id_column = contacts_table.get_column("id").expect("id column");
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "int");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = contacts_table.get_column("email").expect("email column");
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "varchar(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        let contacts_indexes = contacts_table
            .indexes()
            .iter()
            .map(|index| index.name())
            .collect::<Vec<_>>();
        assert!(contacts_indexes[0].contains(&"PK__contacts__".to_string()));

        let users_table = schema.get("users").expect("users table");
        assert_eq!(users_table.name(), "users");
        assert_eq!(users_table.columns().len(), 2);
        let id_column = users_table.get_column("id").expect("id column");
        assert_eq!(id_column.name(), "id");
        assert_eq!(id_column.data_type(), "int");
        assert!(id_column.not_null());
        assert_eq!(id_column.default(), None);
        let email_column = users_table.get_column("email").expect("email column");
        assert_eq!(email_column.name(), "email");
        assert_eq!(email_column.data_type(), "varchar(20)");
        assert!(!email_column.not_null());
        assert_eq!(email_column.default(), None);

        let user_indexes = users_table
            .indexes()
            .iter()
            .map(|index| index.name())
            .collect::<Vec<_>>();
        assert!(user_indexes[0].contains(&"PK__users__".to_string()));

        Ok(())
    }
}
