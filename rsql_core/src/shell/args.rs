use clap::Parser;
use clap_stdin::FileOrStdin;

#[cfg(feature = "driver-rusqlite")]
const DEFAULT_URL: &str = "rusqlite:://?memory=true";

#[cfg(not(feature = "driver-rusqlite"))]
const DEFAULT_URL: &str = "";

#[derive(Debug, Parser)]
pub struct ShellArgs {
    /// The url of the database
    #[arg(short, long, default_value = DEFAULT_URL, env = "DATABASE_URL")]
    pub url: String,

    /// The input file to execute
    #[arg(short, long)]
    pub file: Option<FileOrStdin>,

    /// Sequential list of commands to execute
    #[arg(last = true)]
    pub commands: Vec<String>,
}

impl Default for ShellArgs {
    fn default() -> Self {
        ShellArgs {
            url: DEFAULT_URL.to_string(),
            file: None,
            commands: vec![],
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
        assert!(args.file.is_none());
        let empty_commands: Vec<String> = Vec::new();
        assert_eq!(args.commands, empty_commands);
    }
}
