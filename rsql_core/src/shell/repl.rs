use crate::commands::{CommandManager, LoopCondition};
use crate::configuration::Configuration;
use crate::drivers::{Connection, DriverManager};
use crate::executors::Executor;
use crate::formatters::FormatterManager;
use crate::shell::helper::ReplHelper;
use crate::shell::Result;
use crate::shell::ShellArgs;
use colored::Colorize;
use regex::Regex;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::history::{DefaultHistory, FileHistory};
use rustyline::{ColorMode, CompletionType, Editor};
use std::fmt::Debug;
use std::{fmt, io};
use tracing::error;

/// A builder for creating a [Shell].
#[derive(Debug)]
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

    /// Set the configuration for the shell.
    pub fn with_configuration(mut self, configuration: Configuration) -> Self {
        self.shell.configuration = configuration;
        self
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

    /// Set the formatter manager for the shell.
    pub fn with_output(mut self, output: Box<(dyn io::Write + Send + Sync)>) -> Self {
        self.shell.output = output;
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
    pub configuration: Configuration,
    pub driver_manager: DriverManager,
    pub command_manager: CommandManager,
    pub formatter_manager: FormatterManager,
    pub output: Box<(dyn io::Write + Send + Sync)>,
}

/// Shell implementation.
impl Shell {
    fn new(
        configuration: Configuration,
        driver_manager: DriverManager,
        command_manager: CommandManager,
        formatter_manager: FormatterManager,
        output: Box<(dyn io::Write + Send + Sync)>,
    ) -> Self {
        Self {
            configuration,
            driver_manager,
            command_manager,
            formatter_manager,
            output,
        }
    }

    /// Execute the shell with the provided arguments.
    pub async fn execute(&mut self, args: &ShellArgs) -> Result<()> {
        let mut binding = self
            .driver_manager
            .connect(&self.configuration, args.url.as_str())
            .await?;
        let connection = binding.as_mut();
        let commands = if let Some(file) = &args.file {
            let contents = file.clone().contents()?;
            let mut commands = self.parse_commands(contents).await?;
            commands.extend(args.commands.clone());
            commands
        } else {
            args.commands.clone()
        };

        if commands.is_empty() {
            self.repl(connection).await?;
        } else {
            self.process_commands(connection, commands).await?;
        }

        connection.stop().await?;
        Ok(())
    }

    async fn parse_commands(&mut self, contents: String) -> Result<Vec<String>> {
        let command_identifier = regex::escape(&self.configuration.command_identifier);
        let pattern = format!(r"(?ms)^\s*({}.*?|.*?;)\s*$", command_identifier);
        let regex = Regex::new(pattern.as_str())?;
        let commands: Vec<String> = regex
            .find_iter(contents.as_str())
            .map(|mat| mat.as_str().trim().to_string())
            .collect();
        Ok(commands)
    }

    /// Run with the provided commands.
    async fn process_commands(
        &mut self,
        connection: &mut dyn Connection,
        commands: Vec<String>,
    ) -> Result<()> {
        for command in commands {
            let result = &self
                .evaluate(connection, &DefaultHistory::new(), command.to_string())
                .await;

            match result {
                Ok(LoopCondition::Continue) => {}
                Ok(LoopCondition::Exit(exit_code)) => {
                    if self.configuration.bail_on_error {
                        std::process::exit(*exit_code);
                    }
                }
                Err(error) => {
                    if self.configuration.color {
                        eprintln!("{}: {:?}", "Error".red(), error);
                    } else {
                        eprintln!("Error: {:?}", error);
                    }
                    if self.configuration.bail_on_error {
                        std::process::exit(1);
                    }
                }
            }
        }

        Ok(())
    }

    fn editor(&self, history_file: &str) -> Result<Editor<ReplHelper, FileHistory>> {
        let helper = ReplHelper::new(&self.configuration);
        let mut editor = Editor::<ReplHelper, FileHistory>::new()?;
        if self.configuration.color {
            editor.set_color_mode(ColorMode::Forced);
        } else {
            editor.set_color_mode(ColorMode::Disabled);
        }
        editor.set_edit_mode(self.configuration.edit_mode);
        editor.set_completion_type(CompletionType::Circular);
        editor.set_helper(Some(helper));

        if self.configuration.history {
            let _ = editor.load_history(history_file);
            editor.set_history_ignore_dups(self.configuration.history_ignore_dups)?;

            if self.configuration.history_limit > 0 {
                editor.set_max_history_size(self.configuration.history_limit)?;
            }
        }

        Ok(editor)
    }

    /// Run the Read-Eval-Print Loop (REPL) for the shell.
    async fn repl(&mut self, connection: &mut dyn Connection) -> Result<()> {
        let history_file = match self.configuration.history_file {
            Some(ref file) => String::from(file.to_string_lossy()),
            None => String::new(),
        };

        loop {
            // Create a new editor for each iteration in order to read any changes to the configuration.
            let mut editor = self.editor(history_file.as_str())?;
            let locale = self.configuration.locale.as_str();
            let prompt = t!(
                "prompt",
                locale = locale,
                program_name = self.configuration.program_name,
            );

            let loop_condition = match editor.readline(&prompt) {
                Ok(line) => {
                    let result = &self
                        .evaluate(connection, editor.history(), line.clone())
                        .await
                        .unwrap_or_else(|error| {
                            let mut error_string = "Error:".to_string();
                            let error_message = format!("{:?}", error);
                            if self.configuration.color {
                                error_string = error_string.red().to_string();
                            }
                            eprintln!(
                                "{error}: {message}",
                                error = error_string,
                                message = error_message
                            );
                            if self.configuration.bail_on_error {
                                LoopCondition::Exit(1)
                            } else {
                                LoopCondition::Continue
                            }
                        });
                    let result = result.clone();

                    if result == LoopCondition::Continue && self.configuration.history {
                        let _ = editor.add_history_entry(line.as_str());
                    }
                    result
                }
                Err(ReadlineError::Interrupted) => {
                    let mut program_interrupted =
                        t!("program_interrupted", locale = locale).to_string();
                    if self.configuration.color {
                        program_interrupted = program_interrupted.red().to_string();
                    }
                    eprintln!("{}", program_interrupted);
                    error!("Program interrupted");
                    connection.stop().await?;
                    LoopCondition::Exit(1)
                }
                Err(error) => {
                    let mut error_string = t!("error", locale = locale).to_string();
                    let error_message = format!("{:?}", error);
                    if self.configuration.color {
                        error_string = error_string.red().to_string();
                    }
                    eprintln!(
                        "{}",
                        t!(
                            "error_format",
                            locale = locale,
                            error = error_string.red(),
                            message = error_message,
                        )
                    );
                    error!(error_message);
                    LoopCondition::Exit(1)
                }
            };

            match loop_condition {
                LoopCondition::Continue => {}
                LoopCondition::Exit(exit_code) => {
                    if self.configuration.history {
                        editor.save_history(history_file.as_str())?;
                    }

                    std::process::exit(exit_code);
                }
            }
        }
    }

    /// Evaluate the input line and return the loop condition.
    async fn evaluate(
        &mut self,
        connection: &mut dyn Connection,
        history: &DefaultHistory,
        line: String,
    ) -> Result<LoopCondition> {
        let mut executor = Executor::new(
            &mut self.configuration,
            &self.command_manager,
            &self.driver_manager,
            &self.formatter_manager,
            history,
            connection,
            &mut self.output,
        );
        let loop_condition = executor.execute(line.as_str()).await?;
        Ok(loop_condition)
    }
}

/// Default implementation for [Shell].
impl Default for Shell {
    fn default() -> Self {
        Self::new(
            Configuration::default(),
            DriverManager::default(),
            CommandManager::default(),
            FormatterManager::default(),
            Box::new(io::stdout()),
        )
    }
}

impl Debug for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shell")
            .field("configuration", &self.configuration)
            .field("driver_manager", &self.driver_manager)
            .field("command_manager", &self.command_manager)
            .field("formatter_manager", &self.formatter_manager)
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::drivers::{MockConnection, MockDriver};
    use indoc::indoc;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_shell_builder() {
        let configuration = Configuration {
            bail_on_error: true,
            ..Default::default()
        };
        let driver_manager = DriverManager::new();
        let command_manager = CommandManager::new();
        let formatter_manager = FormatterManager::new();
        let output = Vec::new();
        let shell = ShellBuilder::default()
            .with_configuration(configuration)
            .with_driver_manager(driver_manager)
            .with_command_manager(command_manager)
            .with_formatter_manager(formatter_manager)
            .with_output(Box::new(output))
            .build();

        assert_eq!(shell.configuration.bail_on_error, true);
        assert!(shell.driver_manager.iter().next().is_none());
        assert!(shell.command_manager.iter().next().is_none());
        assert!(shell.formatter_manager.iter().next().is_none());
    }

    #[test]
    fn test_shell_debug() {
        let shell = Shell::default();
        let debug = format!("{:?}", shell);
        assert!(debug.contains("Shell"));
        assert!(debug.contains("configuration"));
        assert!(debug.contains("driver_manager"));
        assert!(debug.contains("command_manager"));
        assert!(debug.contains("formatter_manager"));
    }

    #[tokio::test]
    async fn test_execute() -> anyhow::Result<()> {
        let configuration = Configuration {
            bail_on_error: false,
            ..Default::default()
        };
        let driver_identifier = "test-driver";
        let mut mock_driver = MockDriver::new();
        mock_driver
            .expect_identifier()
            .returning(|| driver_identifier);
        mock_driver.expect_connect().returning(|_, _| {
            let mut mock_connection = MockConnection::new();
            mock_connection.expect_stop().returning(|| Ok(()));
            Ok(Box::new(mock_connection))
        });
        let mut driver_manager = DriverManager::new();
        driver_manager.add(Box::new(mock_driver));
        let mut shell = ShellBuilder::new()
            .with_configuration(configuration)
            .with_driver_manager(driver_manager)
            .build();
        let mut args = ShellArgs::default();
        args.url = driver_identifier.to_string();
        args.commands = vec![".bail on".to_string()];

        shell.execute(&args).await?;

        assert_eq!(shell.configuration.bail_on_error, true);
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_commands_default_command_identifier() -> anyhow::Result<()> {
        let mut shell = Shell::default();
        let contents = indoc! {r#"
            .bail on
            SELECT *
            FROM table;
            .timer on
            INSERT INTO table ...;
            .exit 1
        "#};
        let commands = shell.parse_commands(contents.to_string()).await?;

        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], ".bail on");
        assert_eq!(commands[1], "SELECT *\nFROM table;");
        assert_eq!(commands[2], ".timer on");
        assert_eq!(commands[3], "INSERT INTO table ...;");
        assert_eq!(commands[4], ".exit 1");
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_commands_backslash_command_identifier() -> anyhow::Result<()> {
        let configuration = Configuration {
            command_identifier: "\\".to_string(),
            ..Default::default()
        };
        let mut shell = ShellBuilder::new()
            .with_configuration(configuration)
            .build();
        let contents = indoc! {r#"
            \bail on
            SELECT *
            FROM table;
            \timer on
            INSERT INTO table ...;
            \exit 1
        "#};
        let commands = shell.parse_commands(contents.to_string()).await?;

        assert_eq!(commands.len(), 5);
        assert_eq!(commands[0], "\\bail on");
        assert_eq!(commands[1], "SELECT *\nFROM table;");
        assert_eq!(commands[2], "\\timer on");
        assert_eq!(commands[3], "INSERT INTO table ...;");
        assert_eq!(commands[4], "\\exit 1");
        Ok(())
    }

    #[tokio::test]
    async fn test_process_commands() -> anyhow::Result<()> {
        let configuration = Configuration {
            bail_on_error: false,
            results_timer: false,
            ..Default::default()
        };
        let mut shell = ShellBuilder::new()
            .with_configuration(configuration)
            .build();
        let mut connection = MockConnection::new();
        let commands = vec![".bail on".to_string(), ".timer on".to_string()];

        shell.process_commands(&mut connection, commands).await?;
        assert_eq!(shell.configuration.bail_on_error, true);
        assert_eq!(shell.configuration.results_timer, true);
        Ok(())
    }

    async fn test_editor(color: bool) -> anyhow::Result<()> {
        let configuration = Configuration {
            bail_on_error: false,
            color,
            history: false,
            ..Default::default()
        };
        let shell = ShellBuilder::new()
            .with_configuration(configuration)
            .build();
        let _ = shell.editor("history.txt")?;
        Ok(())
    }

    #[tokio::test]
    async fn test_editor_color_true() -> anyhow::Result<()> {
        test_editor(true).await
    }

    #[tokio::test]
    async fn test_editor_color_false() -> anyhow::Result<()> {
        test_editor(false).await
    }

    #[tokio::test]
    async fn test_evaluate() -> anyhow::Result<()> {
        let configuration = Configuration {
            bail_on_error: false,
            ..Default::default()
        };
        let mut shell = ShellBuilder::new()
            .with_configuration(configuration)
            .build();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();

        let result = shell
            .evaluate(&mut connection, &history, ".bail on".to_string())
            .await?;

        assert_eq!(result, LoopCondition::Continue);
        assert_eq!(shell.configuration.bail_on_error, true);
        Ok(())
    }
}
