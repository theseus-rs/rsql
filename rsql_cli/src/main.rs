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
struct Args {
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
    let mut configuration = ConfigurationBuilder::new(program_name, version)
        .with_config()
        .build();
    let version = full_version(&configuration)?;

    info!("{version} initialized");

    let result = if args.version {
        version::execute(&mut configuration, &mut io::stdout()).await
    } else {
        shell::execute(&mut configuration, &args.shell_args).await
    };

    info!("{version} completed");
    result
}
