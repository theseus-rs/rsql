use crate::configuration::Configuration;
use anyhow::Result;
use std::io;

/// Get the full version of the program (e.g. "rsql/0.0.0 Linux/5.11.0-37-generic/x86_64").
pub fn full_version(configuration: &Configuration) -> Result<String> {
    let program_name = &configuration.program_name;
    let version = &configuration.version;
    let info = os_info::get();
    let os = format!("{}", info.os_type()).replace(' ', "-");
    let os_version = info.version();
    let architecture = info.architecture().unwrap_or("unknown");

    Ok(format!(
        "{program_name}/{version} {os}/{os_version}/{architecture}"
    ))
}

/// Execute the version command and write the version to the provided output.
pub async fn execute(configuration: &mut Configuration, output: &mut dyn io::Write) -> Result<()> {
    let version = full_version(configuration)?;
    writeln!(output, "{version}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
