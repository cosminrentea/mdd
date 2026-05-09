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

/// toc command produces offset/limit annotations on fixture file.
#[test]
fn toc_produces_offset_limit() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["toc", "tests/fixtures/simple.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(offset="))
        .stdout(predicate::str::contains("limit="))
        .stdout(predicate::str::contains("Section One"))
        .stdout(predicate::str::contains("Section Two"));
}

/// toc with a pattern filters results.
#[test]
fn toc_with_pattern_filters() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["toc", "tests/fixtures/simple.md", "Three"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Section Three"))
        .stdout(predicate::str::contains("Section One").not());
}

/// sec command extracts section content.
#[test]
fn sec_extracts_content() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["sec", "tests/fixtures/simple.md", "Section Two"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Section Two"))
        .stdout(predicate::str::contains("code block"));
}

/// sec with no match exits with error.
#[test]
fn sec_no_match_fails() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["sec", "tests/fixtures/simple.md", "NONEXISTENT"])
        .assert()
        .failure();
}

/// at command finds section containing a line.
#[test]
fn at_finds_section() {
    // Line 20 is inside "Section Two" (the code block)
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["at", "tests/fixtures/simple.md", "20"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Section Two"));
}

/// at with breadcrumb on stderr (climbing one level shows parent > child).
#[test]
fn at_shows_breadcrumb() {
    // Line 35 is inside "Nested Subsection" under "Section Three".
    // With level=1 we climb to Section Three, breadcrumb shows ancestry.
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["at", "tests/fixtures/simple.md", "35", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Section Three"))
        .stderr(predicate::str::contains("Section Three"));
}

/// find subcommand still works (stub).
#[test]
fn find_subcommand_responds() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["find", "tests/fixtures/"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}

/// map subcommand still works (stub).
#[test]
fn map_subcommand_responds() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["map", "tests/fixtures/"])
        .assert()
        .success()
        .stderr(predicate::str::contains("not yet implemented"));
}
