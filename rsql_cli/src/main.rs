#![forbid(unsafe_code)]
#[macro_use]
extern crate rust_i18n;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use rsql_core::commands::{help, quit, ShellCommand};
use rsql_core::configuration::ConfigurationBuilder;
use rsql_core::shell::{ShellArgs, ShellBuilder};
use rsql_core::version;
use rsql_core::version::full_version;
use rust_i18n::t;
use std::io;
use tracing::info;

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
    execute(None, &mut io::stdout()).await
}

pub(crate) async fn execute(args: Option<Args>, output: &mut dyn io::Write) -> Result<()> {
    let args = match args {
        Some(args) => args,
        None => {
            let _ = dotenvy::dotenv();
            Args::try_parse()?
        }
    };

    let program_name = "rsql";
    let version = env!("CARGO_PKG_VERSION");
    let mut configuration = ConfigurationBuilder::new(program_name, version)
        .with_config()
        .build();
    let version = full_version(&configuration);

    info!("{version} initialized");

    let result = if args.version {
        version::execute(&mut configuration, output).await
    } else {
        let command_identifier = &configuration.command_identifier;
        let locale = configuration.locale.as_str();
        let banner_version = t!(
            "banner_version",
            locale = locale,
            version = full_version(&configuration)
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
        eprintln!("{}", banner_version);
        eprintln!("{}", banner_message);

        let mut shell = ShellBuilder::default()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_execute_version() -> Result<()> {
        env::set_var("RSQL_LOG_LEVEL", "off");
        let args = Args {
            shell_args: ShellArgs::default(),
            version: true,
        };
        let mut output = Vec::new();

        let result = execute(Some(args), &mut output).await;

        assert!(result.is_ok());
        let version = String::from_utf8(output)?;
        assert!(version.starts_with("rsql/"));
        Ok(())
    }
}
