use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rsql_core::configuration::Configuration;
use semver::Version;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use tracing::debug;

const UPDATE_CHECK_FILE: &str = "last_update_check";

pub fn should_run_update_check(configuration: &Configuration) -> Result<bool> {
    let config_dir = if let Some(path) = &configuration.config_dir {
        path
    } else {
        return Ok(false);
    };

    let file_path = config_dir.join(UPDATE_CHECK_FILE);

    if let Ok(mut file) = File::open(&file_path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents)? > 0 {
            let last_check_time = DateTime::parse_from_rfc3339(&contents)?;
            let now = Utc::now();
            let last_check_time_utc = last_check_time.with_timezone(&Utc);
            if (now - last_check_time_utc) < Duration::hours(24) {
                return Ok(false);
            }
        }
    }

    // Update the last check time
    create_dir_all(config_dir)?;
    let mut file = File::create(&file_path)?;
    let now = Utc::now().to_rfc3339();
    let _ = file.write_all(now.as_bytes());

    Ok(true)
}

pub async fn check_for_newer_version(
    configuration: &Configuration,
    output: &mut dyn Write,
) -> Result<()> {
    if !should_run_update_check(configuration)? {
        return Ok(());
    }

    let current = Version::parse(&configuration.version)?;
    let release = match octocrab::instance()
        .repos("theseus-rs", "rsql")
        .releases()
        .get_latest()
        .await
    {
        Ok(release) => release,
        Err(error) => {
            debug!("Failed to get latest release: {error:?}");
            return Ok(());
        }
    };
    let latest = Version::parse(release.tag_name.trim_start_matches('v'))?;

    if latest > current {
        let locale = configuration.locale.as_str();
        let newer_version = t!(
            "newer_version",
            locale = locale,
            version = latest.to_string()
        );

        writeln!(output, "{}", newer_version)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_should_run_update_check_first_run() -> Result<()> {
        let config_dir = TempDir::new()?.path().to_owned();
        let mut configuration = Configuration::default();
        configuration.config_dir = Some(config_dir);
        assert!(should_run_update_check(&configuration)?);
        Ok(())
    }

    #[test]
    fn test_should_run_update_check_more_than_check_duration() -> Result<()> {
        let config_dir = TempDir::new()?.path().to_owned();
        let file_path = config_dir.join(UPDATE_CHECK_FILE);

        let mut file = File::create(&file_path)?;
        let now = (Utc::now() - Duration::hours(25)).to_rfc3339();
        let _ = file.write_all(now.as_bytes());

        let mut configuration = Configuration::default();
        configuration.config_dir = Some(config_dir);
        assert!(should_run_update_check(&configuration)?);
        Ok(())
    }

    #[test]
    fn test_should_not_run_update_check_no_config_dir() -> Result<()> {
        let mut configuration = Configuration::default();
        configuration.config_dir = None;
        assert!(!should_run_update_check(&configuration)?);
        Ok(())
    }

    #[test]
    fn test_should_not_run_update_check_less_than_check_duration() -> Result<()> {
        let config_dir = TempDir::new()?.path().to_owned();
        let file_path = config_dir.join(UPDATE_CHECK_FILE);

        let mut file = File::create(&file_path)?;
        let now = Utc::now().to_rfc3339();
        let _ = file.write_all(now.as_bytes());

        let mut configuration = Configuration::default();
        configuration.config_dir = Some(config_dir);
        assert!(!should_run_update_check(&configuration)?);
        Ok(())
    }
}
