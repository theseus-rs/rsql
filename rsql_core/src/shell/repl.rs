use crate::commands::{help, CommandManager, LoopCondition, ShellCommand};
use crate::configuration::Configuration;
use crate::drivers::{Connection, DriverManager};
use crate::executors;
use crate::executors::Executor;
use crate::formatters::FormatterManager;
use crate::shell::helper::ReplHelper;
use crate::shell::Result;
use crate::shell::ShellArgs;
use colored::Colorize;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::history::{DefaultHistory, FileHistory};
use rustyline::{ColorMode, CompletionType, Editor};
use std::fmt::Debug;
use std::{fmt, io};
use tracing::error;

/// A builder for creating a [Shell].
#[derive(Debug)]
pub struct ShellBuilder<'a> {
    shell: Shell<'a>,
}

/// A shell for interacting with a database.
impl<'a> ShellBuilder<'a> {
    pub fn new(output: &'a mut (dyn io::Write + Send + Sync)) -> Self {
        Self {
            shell: Shell::new(
                Configuration::default(),
                DriverManager::default(),
                CommandManager::default(),
                FormatterManager::default(),
                output,
            ),
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

    /// Build the shell.
    pub fn build(self) -> Shell<'a> {
        self.shell
    }
}

/// A shell for interacting with a database.
pub struct Shell<'a> {
    pub configuration: Configuration,
    pub driver_manager: DriverManager,
    pub command_manager: CommandManager,
    pub formatter_manager: FormatterManager,
    pub output: &'a mut (dyn io::Write + Send + Sync),
}

/// Shell implementation.
impl<'a> Shell<'a> {
    fn new(
        configuration: Configuration,
        driver_manager: DriverManager,
        command_manager: CommandManager,
        formatter_manager: FormatterManager,
        output: &'a mut (dyn io::Write + Send + Sync),
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
    pub async fn execute(&mut self, args: &ShellArgs) -> Result<i32> {
        let mut binding = self
            .driver_manager
            .connect(&self.configuration, args.url.as_str())
            .await?;
        let connection = binding.as_mut();
        let input = if let Some(file) = &args.file {
            Some(file.clone().contents()?)
        } else if !args.commands.is_empty() {
            Some(args.commands.join("\n"))
        } else {
            None
        };

        let exit_code = if let Some(input) = input {
            match &self
                .evaluate(connection, &DefaultHistory::new(), input.to_string())
                .await?
            {
                LoopCondition::Continue => 0,
                LoopCondition::Exit(exit_code) => *exit_code,
            }
        } else {
            self.repl(connection).await?
        };

        connection.stop().await?;
        Ok(exit_code)
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
    async fn repl(&mut self, connection: &mut dyn Connection) -> Result<i32> {
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
                    let loop_condition = match &self
                        .evaluate(connection, editor.history(), line.clone())
                        .await
                    {
                        Ok(LoopCondition::Continue) => LoopCondition::Continue,
                        Ok(LoopCondition::Exit(exit_code)) => LoopCondition::Exit(*exit_code),
                        Err(_error) => LoopCondition::Exit(1),
                    };

                    if self.configuration.history {
                        let _ = editor.add_history_entry(line.as_str());
                        editor.save_history(history_file.as_str())?;
                    }

                    loop_condition
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

            if let LoopCondition::Exit(exit_code) = loop_condition {
                return Ok(exit_code);
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
        let result = executor.execute(line.as_str()).await;

        if let Err(executors::Error::InvalidCommand { command_name }) = &result {
            if self.invalid_command_help_available(command_name.clone())? {
                return Ok(LoopCondition::Continue);
            }
        }

        match result {
            Ok(loop_condition) => Ok(loop_condition),
            Err(error) => {
                let locale = self.configuration.locale.as_str();
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

                if self.configuration.bail_on_error {
                    Err(error.into())
                } else {
                    Ok(LoopCondition::Continue)
                }
            }
        }
    }

    fn invalid_command_help_available(&mut self, mut invalid_command: String) -> Result<bool> {
        let locale = self.configuration.locale.as_str();
        let mut help_command = help::Command.name(locale);

        if self
            .command_manager
            .get(locale, help_command.as_str())
            .is_none()
        {
            return Ok(false);
        }

        let command_identifier = &self.configuration.command_identifier;
        help_command = format!("{command_identifier}{help_command}");
        invalid_command = format!("{command_identifier}{invalid_command}");

        if self.configuration.color {
            invalid_command = invalid_command.bold().to_string();
            help_command = help_command.bold().to_string();
        }

        let error_message = t!(
            "invalid_command",
            locale = locale,
            invalid_command = invalid_command,
            help_command = help_command,
        );

        let mut error_string = t!("error", locale = locale).to_string();
        if self.configuration.color {
            error_string = error_string.red().to_string();
        }
        writeln!(
            self.output,
            "{}",
            t!(
                "error_format",
                locale = locale,
                error = error_string.red(),
                message = error_message,
            )
        )?;

        Ok(true)
    }
}

impl Debug for Shell<'_> {
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
        let mut output = Vec::new();
        let shell = ShellBuilder::new(&mut output)
            .with_configuration(configuration)
            .with_driver_manager(driver_manager)
            .with_command_manager(command_manager)
            .with_formatter_manager(formatter_manager)
            .build();

        assert_eq!(shell.configuration.bail_on_error, true);
        assert!(shell.driver_manager.iter().next().is_none());
        assert!(shell.command_manager.iter().next().is_none());
        assert!(shell.formatter_manager.iter().next().is_none());
    }

    #[test]
    fn test_shell_debug() {
        let mut output = Vec::new();
        let shell = ShellBuilder::new(&mut output).build();
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
        mock_driver.expect_connect().returning(|_, _, _| {
            let mut mock_connection = MockConnection::new();
            mock_connection.expect_stop().returning(|| Ok(()));
            Ok(Box::new(mock_connection))
        });
        let mut driver_manager = DriverManager::new();
        driver_manager.add(Box::new(mock_driver));
        let mut output = Vec::new();
        let mut shell = ShellBuilder::new(&mut output)
            .with_configuration(configuration)
            .with_driver_manager(driver_manager)
            .build();
        let mut args = ShellArgs::default();
        args.url = format!("{driver_identifier}://");
        args.commands = vec![".bail on".to_string()];

        shell.execute(&args).await?;

        assert_eq!(shell.configuration.bail_on_error, true);
        Ok(())
    }

    async fn test_editor(color: bool) -> anyhow::Result<()> {
        let configuration = Configuration {
            bail_on_error: false,
            color,
            history: false,
            ..Default::default()
        };
        let mut output = Vec::new();
        let shell = ShellBuilder::new(&mut output)
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
        let mut output = Vec::new();
        let mut shell = ShellBuilder::new(&mut output)
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

    async fn test_eval_invalid_command(
        bail: bool,
        command_manager: CommandManager,
    ) -> anyhow::Result<()> {
        let configuration = Configuration {
            bail_on_error: bail,
            ..Default::default()
        };
        let mut output = Vec::new();
        let mut shell = ShellBuilder::new(&mut output)
            .with_configuration(configuration)
            .with_command_manager(command_manager)
            .build();
        let history = DefaultHistory::new();
        let mut connection = MockConnection::new();

        let result = shell
            .evaluate(&mut connection, &history, ".foo".to_string())
            .await;

        if bail {
            assert!(result.is_err());
        } else {
            assert_eq!(result?, LoopCondition::Continue);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_evaluate_invalid_command() -> anyhow::Result<()> {
        test_eval_invalid_command(false, CommandManager::default()).await
    }

    #[tokio::test]
    async fn test_evaluate_invalid_command_help_not_available_no_bail() -> anyhow::Result<()> {
        test_eval_invalid_command(false, CommandManager::new()).await
    }

    #[tokio::test]
    async fn test_evaluate_invalid_command_help_not_available_bail() -> anyhow::Result<()> {
        test_eval_invalid_command(true, CommandManager::new()).await
    }

    #[test]
    fn test_invalid_command_help_available_returns_true() -> anyhow::Result<()> {
        let configuration = Configuration {
            color: true,
            locale: "en".to_string(),
            ..Default::default()
        };
        let mut output = Vec::new();
        let mut shell = ShellBuilder::new(&mut output)
            .with_configuration(configuration)
            .build();
        let invalid_command = "foo".to_string();

        assert!(shell.invalid_command_help_available(invalid_command)?);

        let command_output = String::from_utf8(output).unwrap();
        assert!(command_output.contains(".foo"));
        assert!(command_output.contains(".help"));
        Ok(())
    }

    #[test]
    fn test_invalid_command_help_available_returns_false() -> anyhow::Result<()> {
        let mut output = Vec::new();
        let mut shell = ShellBuilder::new(&mut output)
            .with_command_manager(CommandManager::new())
            .build();
        let invalid_command = "foo".to_string();

        assert!(!shell.invalid_command_help_available(invalid_command)?);
        Ok(())
    }
}
