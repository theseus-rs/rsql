use crate::configuration::Configuration;
use crate::engine::{DriverManager, Engine, QueryResult};
use crate::shell::command::{CommandManager, CommandOptions, LoopCondition};
use crate::shell::repl::display;
use crate::shell::repl::helper::ReplHelper;
use crate::shell::ShellArgs;
use crate::version::full_version;
use anyhow::Result;
use colored::Colorize;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::history::{DefaultHistory, FileHistory};
use rustyline::Editor;
use std::io;
use tracing::error;

pub(crate) enum SqlResult {
    Query(QueryResult),
    Execute(u64),
}

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

pub async fn execute(
    driver_manager: DriverManager,
    command_manager: &CommandManager,
    configuration: &mut Configuration,
    args: &ShellArgs,
) -> Result<()> {
    let mut binding = driver_manager.connect(args.url.as_str()).await?;
    let engine = binding.as_mut();

    repl(command_manager, configuration, engine).await?;

    engine.stop().await
}

async fn repl(
    command_manager: &CommandManager,
    configuration: &mut Configuration,
    engine: &mut dyn Engine,
) -> Result<()> {
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
            Ok(line) => evaluate(command_manager, configuration, engine, &mut editor, line)
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
                engine.stop().await?;
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

async fn evaluate(
    command_manager: &CommandManager,
    configuration: &mut Configuration,
    engine: &mut dyn Engine,
    editor: &mut Editor<ReplHelper, DefaultHistory>,
    line: String,
) -> Result<LoopCondition> {
    let loop_condition = if line.starts_with('.') {
        execute_command(
            command_manager,
            configuration,
            engine,
            editor,
            line.as_str(),
        )
        .await?
    } else {
        execute_sql(configuration, engine, line.as_str()).await?
    };

    if configuration.history {
        let _ = editor.add_history_entry(line.as_str());
    }

    Ok(loop_condition)
}

async fn execute_command(
    command_manager: &CommandManager,
    configuration: &mut Configuration,
    engine: &mut dyn Engine,
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
                engine,
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

async fn execute_sql(
    configuration: &mut Configuration,
    engine: &mut dyn Engine,
    line: &str,
) -> Result<LoopCondition> {
    let start = std::time::Instant::now();
    let sql = line.trim();
    let command = if sql.len() > 6 { &sql[..6] } else { "" }.trim();

    let sql_result = if command.to_lowercase() == "select" {
        SqlResult::Query(engine.query(sql).await?)
    } else {
        SqlResult::Execute(engine.execute(sql).await?)
    };

    let elapsed = start.elapsed();
    display::table(configuration, sql_result, elapsed)?;
    Ok(LoopCondition::Continue)
}
