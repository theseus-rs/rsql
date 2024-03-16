use crate::commands::{CommandManager, CommandOptions, LoopCondition};
use crate::configuration::Configuration;
use crate::drivers::{Connection, DriverManager, Results};
use crate::formatters::{FormatterManager, FormatterOptions};
use crate::shell::helper::ReplHelper;
use crate::shell::ShellArgs;
use crate::version::full_version;
use anyhow::{bail, Result};
use colored::Colorize;
use indicatif::ProgressStyle;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::history::{DefaultHistory, FileHistory};
use rustyline::Editor;
use std::io;
use tracing::{error, instrument, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

/// A builder for creating a [Shell].
pub struct ShellBuilder {
    shell: Shell,
}

/// A shell for interacting with a database.
impl ShellBuilder {
    pub fn new() -> Self {
        Self {
            shell: Shell::default(),
        }
    }

    /// Set the driver manager for the shell.
    pub fn with_driver_manager(mut self, driver_manager: DriverManager) -> Self {
        self.shell.driver_manager = driver_manager;
        self
    }

    /// Set the command manager for the shell.
    pub fn with_command_manager(mut self, command_manager: CommandManager) -> Self {
        self.shell.command_manager = command_manager;
        self
    }

    /// Set the formatter manager for the shell.
    pub fn with_formatter_manager(mut self, formatter_manager: FormatterManager) -> Self {
        self.shell.formatter_manager = formatter_manager;
        self
    }

    /// Set the configuration for the shell.
    pub fn with_configuration(mut self, configuration: Configuration) -> Self {
        self.shell.configuration = configuration;
        self
    }

    /// Build the shell.
    pub fn build(self) -> Shell {
        self.shell
    }
}

/// Default implementation for [ShellBuilder].
impl Default for ShellBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A shell for interacting with a database.
pub struct Shell {
    pub driver_manager: DriverManager,
    pub command_manager: CommandManager,
    pub formatter_manager: FormatterManager,
    pub configuration: Configuration,
}

/// Shell implementation.
impl Shell {
    fn new(
        driver_manager: DriverManager,
        command_manager: CommandManager,
        formatter_manager: FormatterManager,
        configuration: Configuration,
    ) -> Self {
        Self {
            driver_manager,
            command_manager,
            formatter_manager,
            configuration,
        }
    }

    /// Execute the shell with the provided arguments.
    pub async fn execute(&mut self, args: &ShellArgs) -> Result<()> {
        let mut binding = self
            .driver_manager
            .connect(&self.configuration, args.url.as_str())
            .await?;
        let connection = binding.as_mut();

        self.repl(connection).await?;

        connection.stop().await
    }

    /// Run the Read-Eval-Print Loop (REPL) for the shell.
    async fn repl(&mut self, connection: &mut dyn Connection) -> Result<()> {
        let configuration = &mut self.configuration;
        let helper = ReplHelper::new(configuration);
        let history_file = match configuration.history_file {
            Some(ref file) => String::from(file.to_string_lossy()),
            None => String::new(),
        };
        let mut editor = rustyline::Editor::<ReplHelper, FileHistory>::new()?;
        editor.set_color_mode(configuration.color_mode);
        editor.set_edit_mode(configuration.edit_mode);
        editor.set_completion_type(rustyline::CompletionType::Circular);
        editor.set_helper(Some(helper));

        if configuration.history {
            let _ = editor.load_history(history_file.as_str());
            editor.set_history_ignore_dups(configuration.history_ignore_dups)?;

            if configuration.history_limit > 0 {
                editor.set_max_history_size(configuration.history_limit)?;
            }
        }

        welcome_message(configuration)?;
        let prompt = format!("{}> ", configuration.program_name);

        loop {
            let loop_condition = match editor.readline(&prompt) {
                Ok(line) => evaluate(
                    &self.command_manager,
                    configuration,
                    connection,
                    &mut editor,
                    line,
                )
                .await
                .unwrap_or_else(|error| {
                    eprintln!("{}: {:?}", "Error".red(), error);
                    if configuration.bail_on_error {
                        LoopCondition::Exit(1)
                    } else {
                        LoopCondition::Continue
                    }
                }),
                Err(ReadlineError::Interrupted) => {
                    eprintln!("{}", "Program interrupted".red());
                    error!("{}", "Program interrupted".red());
                    connection.stop().await?;
                    LoopCondition::Exit(1)
                }
                Err(error) => {
                    eprintln!("{}: {:?}", "Error".red(), error);
                    error!("{}: {:?}", "Error".red(), error);
                    LoopCondition::Exit(1)
                }
            };

            match loop_condition {
                LoopCondition::Continue => {}
                LoopCondition::Exit(exit_code) => {
                    if configuration.history {
                        editor.save_history(history_file.as_str())?;
                    }

                    std::process::exit(exit_code);
                }
            }
        }
    }
}

/// Default implementation for [Shell].
impl Default for Shell {
    fn default() -> Self {
        Self::new(
            DriverManager::default(),
            CommandManager::default(),
            FormatterManager::default(),
            Configuration::default(),
        )
    }
}

/// Display the welcome message.
fn welcome_message(configuration: &Configuration) -> Result<()> {
    let version = full_version(configuration)?;

    eprintln!("{}", version);
    eprintln!(
        "Type '{}' for help, '{}' to exit.",
        ".help".bold(),
        ".quit".bold()
    );
    Ok(())
}

/// Evaluate the input line and return the loop condition.
async fn evaluate(
    command_manager: &CommandManager,
    configuration: &mut Configuration,
    connection: &mut dyn Connection,
    editor: &mut Editor<ReplHelper, DefaultHistory>,
    line: String,
) -> Result<LoopCondition> {
    let loop_condition = if line.starts_with('.') {
        execute_command(
            command_manager,
            configuration,
            connection,
            editor,
            line.as_str(),
        )
        .await?
    } else {
        execute_sql(configuration, connection, line.as_str()).await?
    };

    if configuration.history {
        let _ = editor.add_history_entry(line.as_str());
    }

    Ok(loop_condition)
}

/// Execute the command and return the loop condition.
async fn execute_command(
    command_manager: &CommandManager,
    configuration: &mut Configuration,
    connection: &mut dyn Connection,
    editor: &mut Editor<ReplHelper, DefaultHistory>,
    line: &str,
) -> Result<LoopCondition> {
    let input: Vec<&str> = line.split_whitespace().collect();
    let output = &mut io::stdout();
    let command_name = &input[0][1..input[0].len()];

    let loop_condition = match command_manager.get(command_name) {
        Some(command) => {
            let history = editor.history();
            let options = CommandOptions {
                command_manager,
                configuration,
                connection,
                history,
                input,
                output,
            };
            command.execute(options).await?
        }
        None => {
            eprintln!("{}: .{command_name}", "Unrecognized command".red());
            if configuration.bail_on_error {
                LoopCondition::Exit(1)
            } else {
                LoopCondition::Continue
            }
        }
    };

    Ok(loop_condition)
}

/// Execute SQL and return the loop condition.
async fn execute_sql(
    configuration: &mut Configuration,
    connection: &mut dyn Connection,
    line: &str,
) -> Result<LoopCondition> {
    let start = std::time::Instant::now();
    let sql = line.trim();
    let command = if sql.len() > 6 { &sql[..6] } else { "" }.trim();

    let results = execute(connection, sql, command).await?;

    let formatter_manager = FormatterManager::default();
    let result_format = &configuration.results_format;
    let formatter = match formatter_manager.get(result_format) {
        Some(formatter) => formatter,
        None => bail!("{}: {}", "Error".red(), "Invalid format"),
    };
    let mut options = FormatterOptions {
        configuration,
        results: &results,
        elapsed: &start.elapsed(),
        output: &mut io::stdout(),
    };
    formatter.format(&mut options).await?;
    Ok(LoopCondition::Continue)
}

/// Execute the SQL and return the results.
#[instrument(skip(connection))]
async fn execute(connection: &mut dyn Connection, sql: &str, command: &str) -> Result<Results> {
    Span::current().pb_set_style(&ProgressStyle::with_template(
        "{span_child_prefix}{spinner}",
    )?);

    if command.to_lowercase() == "select" {
        connection.query(sql).await
    } else {
        connection.execute(sql).await
    }
}
