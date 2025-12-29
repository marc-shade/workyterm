//! Integration tests for WorkyTerm

use std::process::Command;

#[test]
fn test_help_command() {
    // Use the release binary directly to avoid cargo warnings
    let output = Command::new("./target/release/workyterm")
        .arg("--help")
        .output()
        .expect("Failed to execute command - run 'cargo build --release' first");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for Claude Code-style CLI help
    assert!(
        stdout.contains("AI coding assistant"),
        "Expected 'AI coding assistant' in help text, got: {}", stdout
    );
    assert!(
        stdout.contains("--print"),
        "Expected '--print' in help text, got: {}", stdout
    );
}

#[test]
fn test_version_command() {
    // Use the release binary directly
    let output = Command::new("./target/release/workyterm")
        .arg("--version")
        .output()
        .expect("Failed to execute command - run 'cargo build --release' first");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("workyterm") && stdout.contains("0.2"),
        "Expected 'workyterm 0.2.x' in version, got: {}", stdout
    );
}
