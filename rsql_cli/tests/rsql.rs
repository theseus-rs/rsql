use std::process::Command;

#[test]
fn test_version() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("rsql/"));
}
