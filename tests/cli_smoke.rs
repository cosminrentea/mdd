use assert_cmd::Command;
use predicates::prelude::*;

/// Smoke test: `mdd --help` outputs usage information and exits 0.
#[test]
fn help_flag_works() {
    Command::cargo_bin("mdd")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Markdown-Driven Development"));
}

/// Smoke test: `mdd --version` outputs the version string.
#[test]
fn version_flag_works() {
    Command::cargo_bin("mdd")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mdd 0.1.0"));
}

/// Smoke test: each subcommand is reachable (prints "not yet implemented").
#[test]
fn toc_subcommand_responds() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["toc", "tests/fixtures/simple.md"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn find_subcommand_responds() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["find", "tests/fixtures/"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}

#[test]
fn map_subcommand_responds() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["map", "tests/fixtures/"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}
