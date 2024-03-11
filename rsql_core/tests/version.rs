use anyhow::Result;
use rsql_core::configuration::Configuration;
use rsql_core::version::{execute, full_version};

const TEST_PROGRAM_NAME: &str = "test-program";
const TEST_VERSION: &str = "1.2.3";

#[test]
fn test_full_version() -> Result<()> {
    let mut configuration = Configuration::default();
    configuration.program_name = TEST_PROGRAM_NAME.to_string();
    configuration.version = TEST_VERSION.to_string();
    let version_prefix = format!("{TEST_PROGRAM_NAME}/{TEST_VERSION}");
    let version = full_version(&configuration)?;
    assert!(version.starts_with(version_prefix.as_str()));
    Ok(())
}

#[tokio::test]
async fn test_execute() -> Result<()> {
    let mut configuration = Configuration::default();
    configuration.program_name = TEST_PROGRAM_NAME.to_string();
    configuration.version = TEST_VERSION.to_string();
    let mut output = Vec::new();
    execute(&mut configuration, &mut output).await?;
    let version_prefix = format!("{TEST_PROGRAM_NAME}/{TEST_VERSION}");
    let version = String::from_utf8(output)?;
    assert!(version.starts_with(version_prefix.as_str()));
    Ok(())
}
