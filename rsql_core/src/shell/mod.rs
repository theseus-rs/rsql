extern crate colored;

mod completer;
mod helper;
mod highlighter;
mod repl;

use clap::Parser;

pub use repl::execute;

#[cfg(feature = "sqlite")]
const DEFAULT_URL: &str = "sqlite::memory:";

#[cfg(not(feature = "sqlite"))]
const DEFAULT_URL: &str = "";

#[derive(Debug, Parser)]
pub struct ShellArgs {
    /// The url of the database
    #[arg(long, default_value = DEFAULT_URL, env = "DATABASE_URL")]
    pub url: String,
}

impl Default for ShellArgs {
    fn default() -> Self {
        ShellArgs {
            url: DEFAULT_URL.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_default() {
        let args = ShellArgs::default();
        assert_eq!(args.url, DEFAULT_URL);
    }
}
