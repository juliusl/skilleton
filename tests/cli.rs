//! Integration tests for the skilleton CLI commands.

use std::process::Command;

fn skilleton() -> Command {
    Command::new(env!("CARGO_BIN_EXE_skilleton"))
}

#[test]
fn help_exits_zero() {
    let output = skilleton().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Build and validate agent skills"));
}

#[test]
fn init_creates_valid_skill_directory() {
    let dir = tempfile::tempdir().unwrap();
    let skill_path = dir.path().join("new-skill");

    let output = skilleton()
        .args(["init", skill_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "init should exit 0");

    assert!(skill_path.join("skill.toml").exists());
    assert!(skill_path.join("procedures").is_dir());

    // Verify the created skill is loadable
    let check_output = skilleton()
        .args(["check", skill_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        check_output.status.success(),
        "check on init'd skill should pass"
    );
}

#[test]
fn init_on_existing_skill_fails() {
    let dir = tempfile::tempdir().unwrap();
    let skill_path = dir.path().join("existing");

    // First init succeeds
    let output = skilleton()
        .args(["init", skill_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Second init fails
    let output = skilleton()
        .args(["init", skill_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success(), "re-init should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"));
}

#[test]
fn check_valid_fixture_exits_zero() {
    let output = skilleton()
        .args(["check", "tests/fixtures/onboarding"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "check on valid fixture should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_missing_directory_exits_nonzero() {
    let output = skilleton()
        .args(["check", "nonexistent/path"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn build_valid_fixture_outputs_markdown() {
    let output = skilleton()
        .args(["build", "tests/fixtures/onboarding"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "build should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("# Onboarding"), "should start with skill title");
}

#[test]
fn build_output_has_policies_before_procedures() {
    let output = skilleton()
        .args(["build", "tests/fixtures/onboarding"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let policies_pos = stdout.find("## Policies").expect("should have Policies section");
    let procedures_pos = stdout
        .find("## Procedures")
        .expect("should have Procedures section");
    assert!(
        policies_pos < procedures_pos,
        "Policies must appear before Procedures in build output"
    );
}

#[test]
fn build_missing_directory_exits_nonzero() {
    let output = skilleton()
        .args(["build", "nonexistent/path"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty(), "no stdout on failure");
}

#[test]
fn check_invalid_fixture_reports_errors() {
    let output = skilleton()
        .args(["check", "tests/fixtures/invalid"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "check on invalid fixture should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"), "should report errors on stderr");
}

#[test]
fn build_invalid_fixture_no_stdout() {
    let output = skilleton()
        .args(["build", "tests/fixtures/invalid"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "build on invalid fixture should fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty(), "no stdout on validation failure");
}

#[test]
fn init_with_trailing_slash() {
    let dir = tempfile::tempdir().unwrap();
    let path_str = format!("{}/my-skill/", dir.path().display());

    let output = skilleton()
        .args(["init", &path_str])
        .output()
        .unwrap();
    // Should succeed — trailing slash is a valid path
    assert!(
        output.status.success(),
        "init with trailing slash should work: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
