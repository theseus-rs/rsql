use rsql_core::configuration::Configuration;

/// Get the full version of the program (e.g. "rsql/0.0.0 Linux/5.11.0-37-generic/x86_64").
pub fn full_version(configuration: &Configuration) -> String {
    let program_name = &configuration.program_name;
    let version = &configuration.version;
    let info = os_info::get();
    let os = format!("{}", info.os_type()).replace(' ', "-");
    let os_version = info.version();
    let architecture = info.architecture().unwrap_or("unknown");

    format!("{program_name}/{version} {os}/{os_version}/{architecture}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROGRAM_NAME: &str = "test-program";
    const TEST_VERSION: &str = "1.2.3";

    #[test]
    fn test_full_version() -> anyhow::Result<()> {
        let configuration = Configuration {
            program_name: TEST_PROGRAM_NAME.to_string(),
            version: TEST_VERSION.to_string(),
            ..Default::default()
        };
        let version_prefix = format!("{TEST_PROGRAM_NAME}/{TEST_VERSION}");
        let version = full_version(&configuration);
        assert!(version.starts_with(version_prefix.as_str()));
        Ok(())
    }
}
