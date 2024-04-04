#![forbid(unsafe_code)]
#[macro_use]
extern crate rust_i18n;

mod version;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use rsql_core::commands::{help, quit, ShellCommand};
use rsql_core::configuration::{Configuration, ConfigurationBuilder};
use rsql_core::shell::{ShellArgs, ShellBuilder};
use rsql_core::writers::{Output, StdoutWriter};
use rust_i18n::t;
use semver::Version;
use std::io;
use tracing::{debug, info};
use version::full_version;

i18n!("locales", fallback = "en");

#[derive(Debug, Parser)]
pub(crate) struct Args {
    /// The shell arguments
    #[clap(flatten)]
    pub shell_args: ShellArgs,

    /// Display the version of this tool
    #[arg(long)]
    version: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::try_parse()?;

    let program_name = "rsql";
    let version = env!("CARGO_PKG_VERSION");
    let configuration = ConfigurationBuilder::new(program_name, version)
        .with_config()
        .build();
    let output = Output::new(Box::<StdoutWriter>::default());
    let exit_code = execute(args, configuration, output).await?;
    std::process::exit(exit_code);
}

pub(crate) async fn execute(
    args: Args,
    configuration: Configuration,
    mut output: Output,
) -> Result<i32> {
    let version = full_version(&configuration);

    info!("{version} initialized");

    let exit_code = if args.version {
        writeln!(output, "{version}")?;
        0
    } else {
        if args.shell_args.commands.is_empty() && args.shell_args.file.is_none() {
            welcome_message(&configuration, &mut io::stderr()).await?;
        }

        let mut shell = ShellBuilder::default()
            .with_configuration(configuration)
            .with_output(output)
            .build();
        shell.execute(&args.shell_args).await?
    };

    info!("{version} completed");
    Ok(exit_code)
}

async fn welcome_message(configuration: &Configuration, output: &mut dyn io::Write) -> Result<()> {
    let command_identifier = &configuration.command_identifier;
    let locale = configuration.locale.as_str();

    let banner_version = t!(
        "banner_version",
        locale = locale,
        version = full_version(configuration)
    );

    let mut help_command = format!(
        "{command_identifier}{help}",
        command_identifier = command_identifier,
        help = help::Command.name(locale),
    );
    let mut quit_command = format!(
        "{command_identifier}{quit}",
        command_identifier = command_identifier,
        quit = quit::Command.name(locale),
    );
    if configuration.color {
        help_command = help_command.bold().to_string();
        quit_command = quit_command.bold().to_string();
    }

    let banner_message = t!(
        "banner_message",
        locale = locale,
        help_command = help_command,
        quit_command = quit_command
    );

    writeln!(output, "{}", banner_version)?;
    check_for_newer_version(configuration, output).await?;
    writeln!(output, "{}", banner_message)?;
    Ok(())
}

async fn check_for_newer_version(
    configuration: &Configuration,
    output: &mut dyn io::Write,
) -> Result<()> {
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

    #[tokio::test]
    async fn test_execute_version() -> Result<()> {
        let configuration = Configuration {
            program_name: "rsql".to_string(),
            version: "0.0.0".to_string(),
            ..Default::default()
        };
        let args = Args {
            shell_args: ShellArgs::default(),
            version: true,
        };
        let output = Output::default();

        assert_eq!(0, execute(args, configuration, output).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_command() -> Result<()> {
        let configuration = Configuration::default();
        let commands = vec![".exit 42".to_string()];
        let shell_args = ShellArgs {
            commands,
            ..Default::default()
        };
        let args = Args {
            shell_args,
            version: false,
        };
        let output = Output::default();

        assert_eq!(42, execute(args, configuration, output).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_welcome_message() -> Result<()> {
        let mut output = Vec::new();
        let configuration = Configuration {
            program_name: "rsql".to_string(),
            version: "0.0.0".to_string(),
            locale: "en".to_string(),
            color: true,
            command_identifier: ".".to_string(),
            ..Default::default()
        };
        welcome_message(&configuration, &mut output).await?;

        let command_output = String::from_utf8(output).unwrap();
        assert!(command_output.starts_with("rsql/0.0.0"));
        assert!(command_output.contains(".help"));
        assert!(command_output.contains(".quit"));
        Ok(())
    }
}
