use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_flag_works() {
    Command::cargo_bin("mdd")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Markdown-Driven Development"));
}

#[test]
fn version_flag_works() {
    Command::cargo_bin("mdd")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mdd 0.1.0"));
}

#[test]
fn toc_human_produces_offset_limit() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "human", "toc", "tests/fixtures/simple.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("(offset="))
        .stdout(predicate::str::contains("limit="))
        .stdout(predicate::str::contains("Section One"))
        .stdout(predicate::str::contains("Section Two"));
}

#[test]
fn toc_agent_format() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "toc", "tests/fixtures/simple.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[tests/fixtures/simple.md:"))
        .stdout(predicate::str::contains("# Simple Test Document"))
        .stdout(predicate::str::contains("## Section One"));
}

#[test]
fn toc_json_format() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "toc", "tests/fixtures/simple.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"sections\""))
        .stdout(predicate::str::contains("\"offset\""))
        .stdout(predicate::str::contains("\"limit\""));
}

#[test]
fn toc_with_pattern_filters() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "human", "toc", "tests/fixtures/simple.md", "Three"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Section Three"))
        .stdout(predicate::str::contains("Section One").not());
}

#[test]
fn sec_extracts_content() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "human", "sec", "tests/fixtures/simple.md", "Section Two"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Section Two"))
        .stdout(predicate::str::contains("code block"));
}

#[test]
fn sec_no_match_fails() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["sec", "tests/fixtures/simple.md", "NONEXISTENT"])
        .assert()
        .failure();
}

#[test]
fn at_finds_section() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "human", "at", "tests/fixtures/simple.md", "20"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Section Two"));
}

#[test]
fn at_shows_breadcrumb_human() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "human", "at", "tests/fixtures/simple.md", "35", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Section Three"))
        .stderr(predicate::str::contains("Section Three"));
}

#[test]
fn at_agent_format() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "at", "tests/fixtures/simple.md", "20"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[tests/fixtures/simple.md:"))
        .stdout(predicate::str::contains("## Section Two"));
}

#[test]
fn sec_agent_format() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "sec", "tests/fixtures/simple.md", "Section Two"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[tests/fixtures/simple.md:"))
        .stdout(predicate::str::contains("## Section Two"));
}

#[test]
fn find_lists_files() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "find", "tests/fixtures/"])
        .assert()
        .success()
        .stdout(predicate::str::contains("simple.md"))
        .stderr(predicate::str::contains("files matched"));
}

#[test]
fn find_filters_by_table() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "find", "tests/fixtures/", "--has-table"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Section Three"))
        .stdout(predicate::str::contains("no-frontmatter").not());
}

#[test]
fn find_filters_by_type() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "find", "tests/fixtures/", "-t", "reference"])
        .assert()
        .success()
        .stdout(predicate::str::contains("simple.md"));
}

#[test]
fn map_shows_overview() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["map", "tests/fixtures/"])
        .assert()
        .success()
        .stdout(predicate::str::contains("=== fixtures/"))
        .stdout(predicate::str::contains("TOTAL:"));
}
