use anyhow::Result;
use axoupdater::AxoUpdater;
use chrono::{DateTime, Duration, Utc};
use inquire::ui::RenderConfig;
use inquire::Confirm;
use rsql_core::configuration::Configuration;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};

const UPDATE_CHECK_FILE: &str = "last_update_check";

pub fn should_run_update_check(configuration: &Configuration) -> Result<bool> {
    let Some(config_dir) = &configuration.config_dir else {
        return Ok(false);
    };

    let file_path = config_dir.join(UPDATE_CHECK_FILE);

    if let Ok(mut file) = File::open(&file_path) {
        let mut contents = String::new();
        if file.read_to_string(&mut contents)? > 0 {
            let last_check_time = DateTime::parse_from_rfc3339(contents.trim())?;
            let now = Utc::now();
            let last_check_time_utc = last_check_time.with_timezone(&Utc);
            if (now - last_check_time_utc) < Duration::weeks(1) {
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

    let mut updater = AxoUpdater::new_for("rsql_cli");
    updater.disable_installer_output();
    let receipt = updater.load_receipt()?;
    if receipt.is_update_needed().await? {
        let locale = configuration.locale.as_str();
        let no = t!("no", locale = locale).to_lowercase();
        let yes = t!("yes", locale = locale).to_string();
        let update_version_prompt = t!("update_version_prompt", locale = locale);
        let update_version_prompt_placeholder = t!(
            "update_version_prompt_placeholder",
            locale = locale,
            yes = yes,
            no = no,
        );
        let update_version_prompt_error = t!(
            "update_version_prompt_error",
            locale = locale,
            yes = yes,
            no = no,
        );

        let confirm = Confirm {
            message: &update_version_prompt,
            starting_input: None,
            default: Some(true),
            placeholder: Some(&*update_version_prompt_placeholder),
            help_message: None,
            formatter: &|answer| {
                if answer {
                    yes.to_string()
                } else {
                    no.to_string()
                }
            },
            parser: &|answer| Ok(answer == yes),
            error_message: update_version_prompt_error.to_string(),
            default_value_formatter: &|default| {
                if default {
                    yes.to_string()
                } else {
                    no.to_string()
                }
            },
            render_config: RenderConfig::default(),
        };

        if !confirm.prompt()? {
            return Ok(());
        }

        let update_result = receipt.run().await?;
        if let Some(result) = update_result {
            let version = result.new_version;
            let updated_version = t!("updated_version", locale = locale, version = version,);
            writeln!(output, "{updated_version}")?;
        }
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
        let configuration = Configuration {
            config_dir: Some(config_dir),
            ..Default::default()
        };
        assert!(should_run_update_check(&configuration)?);
        Ok(())
    }

    #[test]
    fn test_should_run_update_check_more_than_check_duration() -> Result<()> {
        let config_dir = TempDir::new()?.path().to_owned();
        let file_path = config_dir.join(UPDATE_CHECK_FILE);

        create_dir_all(&config_dir)?;
        let mut file = File::create(file_path)?;
        let now = (Utc::now() - Duration::weeks(2)).to_rfc3339();
        let _ = file.write_all(now.as_bytes());

        let configuration = Configuration {
            config_dir: Some(config_dir),
            ..Default::default()
        };
        assert!(should_run_update_check(&configuration)?);
        Ok(())
    }

    #[test]
    fn test_should_not_run_update_check_no_config_dir() -> Result<()> {
        let configuration = Configuration {
            config_dir: None,
            ..Default::default()
        };
        assert!(!should_run_update_check(&configuration)?);
        Ok(())
    }

    #[test]
    fn test_should_not_run_update_check_less_than_check_duration() -> Result<()> {
        let config_dir = TempDir::new()?.path().to_owned();
        let file_path = config_dir.join(UPDATE_CHECK_FILE);

        create_dir_all(&config_dir)?;
        let mut file = File::create(file_path)?;
        let now = Utc::now().to_rfc3339();
        let _ = file.write_all(now.as_bytes());

        let configuration = Configuration {
            config_dir: Some(config_dir),
            ..Default::default()
        };
        assert!(!should_run_update_check(&configuration)?);
        Ok(())
    }
}
