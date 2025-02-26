#![forbid(unsafe_code)]
#![deny(clippy::pedantic)]
use rsql_drivers::{DriverManager, Result};

/// Example of using the DriverManager to connect to a database.
#[tokio::main]
async fn main() -> Result<()> {
    let database_url = "sqlite://";
    let driver_manager = DriverManager::default();
    let mut connection = driver_manager.connect(database_url).await?;

    let _ = connection
        .execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
        .await?;

    let _ = connection
        .execute("INSERT INTO users (id, name) VALUES (1, 'John Doe')")
        .await?;
    let _ = connection
        .execute("INSERT INTO users (id, name) VALUES (2, 'Jane Smith')")
        .await?;

    let mut query_result = connection
        .query("SELECT id, name FROM users ORDER BY id")
        .await?;

    let columns = query_result.columns().await;
    let column_names = columns.join(", ");
    println!("{column_names}");

    while let Some(row) = query_result.next().await {
        let values = row.iter().map(ToString::to_string).collect::<Vec<String>>();
        let row_data = values.join(", ");
        println!("{row_data}");
    }

    connection.close().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_main() -> Result<()> {
        main()
    }
}
