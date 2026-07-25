#![cfg(all(unix, feature = "driver-rusqlite"))]
#![expect(
    clippy::panic_in_result_fn,
    reason = "test assertions intentionally panic when verification fails"
)]

use expectrl::{ControlCode, Eof, Expect, Session};
use rsql_core::{Configuration, EditMode};
use rsql_repl::shell::{ShellArgs, ShellBuilder};
use std::env;
use std::process::Command;
use std::time::Duration;

const EXIT_CODE_OUTPUT: &str = "REPL_EXIT_CODE=0";

fn assert_ctrl_c_exits_repl(child_test: &str) -> anyhow::Result<()> {
    let mut command = Command::new(env::current_exe()?);
    command
        .arg("--ignored")
        .arg("--exact")
        .arg(child_test)
        .arg("--nocapture");

    let mut session = Session::spawn(command)?;
    session.set_expect_timeout(Some(Duration::from_secs(30)));

    let _ = session.expect("rsql> ")?;
    session.send(ControlCode::EndOfText)?;
    let _ = session.expect(EXIT_CODE_OUTPUT)?;
    let _ = session.expect(Eof)?;

    Ok(())
}

#[test]
fn ctrl_c_exits_repl_in_emacs_mode() -> anyhow::Result<()> {
    assert_ctrl_c_exits_repl("run_repl_child_in_emacs_mode")
}

#[test]
fn ctrl_c_exits_repl_in_vi_mode() -> anyhow::Result<()> {
    assert_ctrl_c_exits_repl("run_repl_child_in_vi_mode")
}

#[test]
#[ignore = "spawned in a pseudo-terminal by ctrl_c_exits_repl_in_emacs_mode"]
fn run_repl_child_in_emacs_mode() -> anyhow::Result<()> {
    run_repl_child(EditMode::Emacs)
}

#[test]
#[ignore = "spawned in a pseudo-terminal by ctrl_c_exits_repl_in_vi_mode"]
fn run_repl_child_in_vi_mode() -> anyhow::Result<()> {
    run_repl_child(EditMode::Vi)
}

fn run_repl_child(edit_mode: EditMode) -> anyhow::Result<()> {
    let configuration = Configuration {
        color: false,
        edit_mode,
        history: false,
        program_name: "rsql".to_string(),
        ..Default::default()
    };
    let args = ShellArgs {
        url: "rusqlite://".to_string(),
        ..Default::default()
    };
    let mut shell = ShellBuilder::default()
        .with_configuration(configuration)
        .build();

    let exit_code = tokio::runtime::Runtime::new()?.block_on(shell.execute(&args))?;
    println!("REPL_EXIT_CODE={exit_code}");
    assert_eq!(exit_code, 0);

    Ok(())
}
