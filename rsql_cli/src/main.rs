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
use rust_i18n::t;
use std::io;
use tracing::info;
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

    execute(args, configuration, &mut io::stdout()).await
}

pub(crate) async fn execute(
    args: Args,
    configuration: Configuration,
    output: &mut (dyn io::Write + Send + Sync),
) -> Result<()> {
    let version = full_version(&configuration);

    info!("{version} initialized");

    let result = if args.version {
        writeln!(output, "{version}")?;
        Ok(())
    } else {
        if args.shell_args.commands.is_empty() && args.shell_args.file.is_none() {
            welcome_message(&mut io::stderr(), &configuration);
        }

        let mut shell = ShellBuilder::new(output)
            .with_configuration(configuration)
            .build();
        shell.execute(&args.shell_args).await
    };

    info!("{version} completed");

    match result {
        Ok(_) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn welcome_message(output: &mut dyn io::Write, configuration: &Configuration) {
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

    writeln!(output, "{}", banner_version).expect("failed to banner version");
    writeln!(output, "{}", banner_message).expect("failed to banner message")
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

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
        let mut output = Vec::new();

        let _ = execute(args, configuration, &mut output).await?;

        let version = String::from_utf8(output)?;
        assert!(version.starts_with("rsql/0.0.0"));
        Ok(())
    }

    #[tokio::test]
    async fn test_execute_command() -> Result<()> {
        let configuration = Configuration::default();
        let commands = vec![".locale en".to_string(), ".locale".to_string()];
        let shell_args = ShellArgs {
            commands,
            ..Default::default()
        };
        let args = Args {
            shell_args,
            version: false,
        };
        let mut output = Vec::new();

        let _ = execute(args, configuration, &mut output).await?;

        let comand_output = String::from_utf8(output)?;
        let expected = indoc! {r#"
            Locale: en
        "#};
        assert_eq!(comand_output, expected);
        Ok(())
    }

    #[test]
    fn test_welcome_message() {
        let mut output = Vec::new();
        let configuration = Configuration {
            program_name: "rsql".to_string(),
            version: "0.0.0".to_string(),
            locale: "en".to_string(),
            color: false,
            command_identifier: ".".to_string(),
            ..Default::default()
        };
        welcome_message(&mut output, &configuration);

        let comand_output = String::from_utf8(output).unwrap();
        assert!(comand_output.starts_with("rsql/0.0.0"));
        assert!(comand_output.contains("Type '.help' for help, '.quit' to exit."));
    }
}
