use rsql_driver::{Driver, Result, Value};
use std::path;
use std::path::PathBuf;

/// Returns the url to the specified dataset file.
pub fn dataset_url<S: AsRef<str>>(scheme: S, file_name: S) -> String {
    let scheme = scheme.as_ref();
    let file_name = file_name.as_ref();
    let crate_directory = env!("CARGO_MANIFEST_DIR");
    let mut path = PathBuf::from(crate_directory);
    path.push("..");

    if path.join("datasets").exists() {
        path.push("datasets");
    } else {
        path.push("..");
        path.push("datasets");
    }

    path.push(file_name);

    let dataset_path = path
        .to_string_lossy()
        .to_string()
        .replace(path::MAIN_SEPARATOR, "/");
    #[cfg(target_os = "windows")]
    let dataset_path = if dataset_path.is_empty() {
        dataset_path
    } else {
        format!("/{dataset_path}")
    };

    format!("{scheme}://{dataset_path}")
}

#[tokio::test]
async fn test_json_metadata() -> Result<()> {
    let database_url = dataset_url("json", "cheyenne.json");
    let driver = rsql_driver_json::Driver;
    let mut connection = driver.connect(&database_url).await?;

    let mut query_result = connection
        .query(
            r#"
            SELECT geometry.coordinates[1] AS longitude,
                   geometry.coordinates[2] AS latitude
              FROM cheyenne
        "#,
        )
        .await?;

    assert_eq!(query_result.columns().await, vec!["longitude", "latitude"]);
    assert_eq!(
        query_result.next().await,
        Some(vec![Value::F64(-104.820246), Value::F64(41.139981)])
    );
    assert!(query_result.next().await.is_none());

    let metadata = connection.metadata().await?;
    let tables = metadata.current_schema().expect("schema").tables();
    assert_eq!(tables.len(), 1);
    let cheyenne_table = tables[0];
    assert_eq!(cheyenne_table.name(), "cheyenne");
    assert_eq!(cheyenne_table.columns().len(), 6);

    let type_column = cheyenne_table.get_column("type").expect("type column");
    assert_eq!(type_column.name(), "type");
    assert_eq!(type_column.data_type(), "str");
    assert!(!type_column.not_null());
    assert_eq!(type_column.default(), None);

    let geometry_column = cheyenne_table
        .get_column("geometry")
        .expect("geometry column");
    assert_eq!(geometry_column.name(), "geometry");
    assert_eq!(geometry_column.data_type(), "struct[2]");
    assert!(!geometry_column.not_null());
    assert_eq!(geometry_column.default(), None);

    let geo_type_column = cheyenne_table
        .get_column("geometry.type")
        .expect("geometry.type column");
    assert_eq!(geo_type_column.name(), "geometry.type");
    assert_eq!(geo_type_column.data_type(), "str");
    assert!(!geo_type_column.not_null());
    assert_eq!(geo_type_column.default(), None);

    let geo_coordinates_column = cheyenne_table
        .get_column("geometry.coordinates")
        .expect("geometry.coordinates column");
    assert_eq!(geo_coordinates_column.name(), "geometry.coordinates");
    assert_eq!(geo_coordinates_column.data_type(), "list[f64]");
    assert!(!geo_coordinates_column.not_null());
    assert_eq!(geo_coordinates_column.default(), None);

    let properties_column = cheyenne_table
        .get_column("properties")
        .expect("properties column");
    assert_eq!(properties_column.name(), "properties");
    assert_eq!(properties_column.data_type(), "struct[1]");
    assert!(!properties_column.not_null());
    assert_eq!(properties_column.default(), None);

    let properties_name_column = cheyenne_table
        .get_column("properties.name")
        .expect("properties.name column");
    assert_eq!(properties_name_column.name(), "properties.name");
    assert_eq!(properties_name_column.data_type(), "str");
    assert!(!properties_name_column.not_null());
    assert_eq!(properties_name_column.default(), None);

    connection.close().await?;
    Ok(())
}
