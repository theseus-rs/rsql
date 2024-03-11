#![forbid(unsafe_code)]

use anyhow::Result;
use clap::Parser;
use rsql_core::configuration::ConfigurationBuilder;
use rsql_core::shell::ShellArgs;
use rsql_core::version::full_version;
use rsql_core::{shell, version};
use std::io;
use tracing::info;

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

    execute(&args, &mut io::stdout()).await
}

pub(crate) async fn execute(args: &Args, output: &mut dyn io::Write) -> Result<()> {
    let program_name = "rsql";
    let version = env!("CARGO_PKG_VERSION");
    let mut configuration = ConfigurationBuilder::new(program_name, version)
        .with_config()
        .build();
    let version = full_version(&configuration)?;

    info!("{version} initialized");

    let result = if args.version {
        version::execute(&mut configuration, output).await
    } else {
        shell::execute(&mut configuration, &args.shell_args).await
    };

    info!("{version} completed");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_main() {
        env::set_var("RSQL_LOG_LEVEL", "off");

        let result = main();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_version() -> Result<()> {
        env::set_var("RSQL_LOG_LEVEL", "off");
        let args = Args {
            shell_args: ShellArgs::default(),
            version: true,
        };
        let mut output = Vec::new();

        let result = execute(&args, &mut output).await;

        assert!(result.is_ok());
        let version = String::from_utf8(output)?;
        assert!(version.starts_with("rsql/"));
        Ok(())
    }
}
