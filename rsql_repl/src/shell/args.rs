use clap::Parser;
use clap_stdin::FileOrStdin;

#[cfg(feature = "driver-rusqlite")]
const DEFAULT_URL: &str = "rusqlite://";

#[cfg(not(feature = "driver-rusqlite"))]
const DEFAULT_URL: &str = "";

#[derive(Debug, Parser)]
pub struct ShellArgs {
    /// The output format
    #[arg(long)]
    pub format: Option<String>,

    /// Display the header
    #[arg(long)]
    pub header: Option<bool>,

    /// Display the footer
    #[arg(long)]
    pub footer: Option<bool>,

    /// Display the execution time
    #[arg(long)]
    pub timer: Option<bool>,

    /// Display the rows
    #[arg(long)]
    pub rows: Option<bool>,

    /// Display the changes
    #[arg(long)]
    pub changes: Option<bool>,

    /// Limit the number of rows returned
    #[arg(long)]
    pub limit: Option<usize>,

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
            format: None,
            header: None,
            footer: None,
            timer: None,
            rows: None,
            changes: None,
            limit: None,
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
        assert_eq!(args.format, None);
        assert_eq!(args.header, None);
        assert_eq!(args.footer, None);
        assert_eq!(args.timer, None);
        assert_eq!(args.rows, None);
        assert_eq!(args.changes, None);
        assert_eq!(args.limit, None);
        assert_eq!(args.url, DEFAULT_URL);
        assert!(args.file.is_none());
        let empty_commands: Vec<String> = Vec::new();
        assert_eq!(args.commands, empty_commands);
    }
}
