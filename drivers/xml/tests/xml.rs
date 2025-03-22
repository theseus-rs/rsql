use indoc::indoc;
use rsql_driver::{Driver, Result, Value};
use rsql_driver_test_utils::dataset_url;

fn database_url() -> String {
    dataset_url("xml", "users.xml")
}

#[tokio::test]
async fn test_driver_connect() -> Result<()> {
    let database_url = database_url();
    let driver = rsql_driver_xml::Driver;
    let mut connection = driver.connect(&database_url).await?;
    assert_eq!(&database_url, connection.url());
    connection.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_connection_interface() -> Result<()> {
    let database_url = database_url();
    let driver = rsql_driver_xml::Driver;
    let mut connection = driver.connect(&database_url).await?;

    let mut query_result = connection
        .query(indoc! {r"
                WITH cte_user AS (
                    SELECT unnest(data.user) FROM users
                )
                SELECT user.* FROM cte_user
            "})
        .await?;

    assert_eq!(query_result.columns().await, vec!["id", "name"]);
    assert_eq!(
        query_result.next().await,
        Some(vec![Value::I64(1), Value::String("John Doe".to_string())])
    );
    assert_eq!(
        query_result.next().await,
        Some(vec![Value::I64(2), Value::String("Jane Smith".to_string())])
    );
    assert!(query_result.next().await.is_none());

    connection.close().await?;
    Ok(())
}

#[test]
fn test_xml_to_json() -> Result<()> {
    let xml = indoc! {r#"
        <data foo="42">
            <user score="1.234">
                <id>1</id>
                <name>John Doe</name>
                <email secure="false">john.doe@none.com</email>
            </user>
            <user>
                <name>Jane Smith</name>
                <id>2</id>
            </user>
        </data>
    "#};

    let value = rsql_driver_xml::xml_to_json(xml)?;
    let _json = serde_json::to_string(&value);
    let data = value.get("data").expect("Expected data object");
    let foo = data.get("@foo").expect("Expected foo attribute");
    let foo_value = foo
        .as_i64()
        .expect("Expected foo attribute to be an integer");
    assert_eq!(foo_value, 42);
    let user = data.get("user").expect("Expected user value");
    let user = user.as_array().expect("Expected user value to be an array");
    assert_eq!(user.len(), 2);
    let user1 = user.first().expect("Expected user 1");
    let score = user1
        .get("@score")
        .expect("Expected score attribute")
        .as_f64()
        .expect("Expected score attribute to be a float");
    let diff = score - 1.234;
    assert!(diff.abs() < 0.01f64);
    let user1_id = user1
        .get("id")
        .expect("Expected id")
        .as_i64()
        .expect("Expected id to be an integer");
    let user1_name = user1
        .get("name")
        .expect("Expected name")
        .as_str()
        .expect("Expected name to be a string");
    assert_eq!(user1_id, 1);
    assert_eq!(user1_name, "John Doe");

    // Test element with text and an attribute
    let user1_email = user1.get("email").expect("Expected email");
    let user1_email_secure = user1_email
        .get("@secure")
        .expect("Expected secure attribute")
        .as_bool()
        .expect("Expected secure attribute to be a boolean");
    let user1_email = user1_email
        .get("#text")
        .expect("Expected email text")
        .as_str()
        .expect("Expected email text to be a string");
    assert!(!user1_email_secure);
    assert_eq!(user1_email, "john.doe@none.com");

    let user2 = user.last().expect("Expected user 2");
    let user2_id = user2
        .get("id")
        .expect("Expected id")
        .as_i64()
        .expect("Expected id to be an integer");
    let user2_name = user2
        .get("name")
        .expect("Expected name")
        .as_str()
        .expect("Expected name to be a string");
    assert_eq!(user2_id, 2);
    assert_eq!(user2_name, "Jane Smith");
    Ok(())
}
