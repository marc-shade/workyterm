//! Integration tests for WorkyTerm

use std::process::Command;

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A friendly CLI for non-programmers"));
    assert!(stdout.contains("--task"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--config"));
}

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("workyterm"));
    assert!(stdout.contains("0.1.0"));
}
