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

// ─── agg tests ───────────────────────────────────────────────────────────────

#[test]
fn agg_filters_by_heading() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "agg", "tests/fixtures/", "-H", "Section One"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Section One"))
        .stdout(predicate::str::contains("Item 1"));
}

#[test]
fn agg_respects_budget() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "agg", "tests/fixtures/", "-H", "Section", "--budget", "5"])
        .assert()
        .success()
        .stderr(predicate::str::contains("truncated"));
}

#[test]
fn agg_empty_no_match() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["agg", "tests/fixtures/", "-H", "NONEXISTENT_HEADING_XYZ"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No sections matched"));
}

// ─── links tests ─────────────────────────────────────────────────────────────

#[test]
fn links_lists_from_file() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", "tests/fixtures/links-test.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Markdown"))
        .stdout(predicate::str::contains("WikiLink"));
}

#[test]
fn links_check_finds_broken() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", "tests/fixtures/links-test.md", "--broken"])
        .assert()
        .success()
        .stdout(predicate::str::contains("BROKEN"));
}

#[test]
fn links_skips_pure_anchors() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", "tests/fixtures/links-test.md", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#standard-links").not());
}

// ─── parts tests ─────────────────────────────────────────────────────────────

#[test]
fn parts_lists_entries() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "parts", "tests/fixtures/lrn-two-entries.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Entry 1"))
        .stdout(predicate::str::contains("Entry 2"));
}

#[test]
fn parts_shows_correct_heading_level() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "parts", "tests/fixtures/lrn-two-entries.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("{### First Entry}"))
        .stdout(predicate::str::contains("{### Second Entry}"));
}

#[test]
fn parts_filter_by_type() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "parts", "tests/fixtures/lrn-two-entries.md", "--filter", "type=note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Entry 2"))
        .stdout(predicate::str::contains("Entry 1").not());
}

// ─── stats tests ─────────────────────────────────────────────────────────────

#[test]
fn stats_shows_counts() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["stats", "tests/fixtures/"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Files:"))
        .stdout(predicate::str::contains("Frontmatter:"))
        .stdout(predicate::str::contains("Content:"));
}
