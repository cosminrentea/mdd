#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use std::path::Path;

// Exit codes are grep-like: 0 = found / report succeeded, 1 = nothing found
// (one stderr line `mdd: no match: <detail>`), 2 = error.

fn mdd(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mdd").unwrap().args(args).assert()
}

fn mdd_in(path: &Path, args: &[&str], rest: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(args)
        .arg(path)
        .args(rest)
        .assert()
}

fn write(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

fn no_match(detail: &str) -> String {
    format!("mdd: no match: {detail}\n")
}

// ─── outline and extract ──────────────────────────────────────────────

#[test]
fn toc_pattern_without_match_exits_1_for_a_file_and_a_directory() {
    mdd(&["toc", "tests/fixtures/simple.md", "NOPE_XYZ"])
        .code(1)
        .stdout("")
        .stderr(no_match("no matching sections for pattern: NOPE_XYZ"));
    mdd(&["toc", "tests/fixtures", "NOPE_XYZ"])
        .code(1)
        .stdout("")
        .stderr(no_match("no matching sections for pattern: NOPE_XYZ"));
}

#[test]
fn toc_pattern_on_an_empty_directory_reports_the_pattern() {
    let tmp = tempfile::tempdir().unwrap();
    mdd_in(tmp.path(), &["toc"], &["NOPE_XYZ"])
        .code(1)
        .stdout("")
        .stderr(no_match("no matching sections for pattern: NOPE_XYZ"));
}

#[test]
fn toc_file_without_headings_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write(tmp.path(), "plain.md", "just text\n");
    mdd_in(&path, &["toc"], &[])
        .code(1)
        .stdout("")
        .stderr(no_match(&format!(
            "no headings found in {}",
            path.display()
        )));
}

#[test]
fn toc_on_a_directory_without_markdown_files_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    mdd_in(tmp.path(), &["toc"], &[]).code(1).stderr(format!(
        "\n0 files\n{}",
        no_match(&format!("no Markdown files in {}", tmp.path().display()))
    ));
}

#[test]
fn sec_without_any_match_exits_1() {
    mdd(&["sec", "tests/fixtures/simple.md", "NOPE_XYZ", "NOPE_ABC"])
        .code(1)
        .stdout("")
        .stderr(no_match(
            "no matching sections for pattern: NOPE_XYZ, NOPE_ABC",
        ));
}

#[test]
fn sec_with_a_partial_match_warns_and_exits_0() {
    mdd(&["sec", "tests/fixtures/simple.md", "NOPE_XYZ", "Section One"])
        .code(0)
        .stderr("mdd: warning: no match for: NOPE_XYZ\n");
}

#[test]
fn at_before_the_first_heading_exits_1() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write(tmp.path(), "intro.md", "intro\n\n# H\n\ntext\n");
    mdd_in(&path, &["at"], &["1"])
        .code(1)
        .stdout("")
        .stderr(no_match("line 1 is before any heading"));
    mdd_in(&path, &["at"], &["3"]).code(0);
}

// ─── directory queries ────────────────────────────────────────────────

#[test]
fn find_without_a_result_exits_1() {
    mdd(&["find", "tests/fixtures", "-H", "nomatchxyz"])
        .code(1)
        .stdout("")
        .stderr(no_match("no files matched the given filters"));
}

#[test]
fn find_with_a_result_exits_0_and_keeps_its_count_line() {
    mdd(&["find", "tests/fixtures", "-t", "reference"])
        .code(0)
        .stderr(predicates::str::ends_with(" files matched\n"));
}

#[test]
fn agg_without_a_result_exits_1() {
    mdd(&["agg", "tests/fixtures", "-H", "nomatchxyz"])
        .code(1)
        .stdout("")
        .stderr(no_match("no sections matched the given filters"));
}

#[test]
fn parts_filter_without_a_match_exits_1_and_replaces_the_totals_line() {
    mdd(&[
        "parts",
        "tests/fixtures/multipart-lrn.md",
        "--filter",
        "type=zzz",
    ])
    .code(1)
    .stdout("")
    .stderr(no_match("0 entries shown (of 3 total)"));
}

#[test]
fn parts_with_a_match_exits_0_and_keeps_its_totals_line() {
    mdd(&["parts", "tests/fixtures/multipart-lrn.md"])
        .code(0)
        .stderr("3 entries shown (of 3 total)\n");
}

// ─── frontmatter ──────────────────────────────────────────────────────

#[test]
fn fm_list_values_without_a_value_exits_1() {
    mdd(&["fm", "tests/fixtures", "--list-values", "nosuchfield"])
        .code(1)
        .stdout("")
        .stderr(no_match("no values found for field 'nosuchfield'"));
}

#[test]
fn fm_file_without_frontmatter_exits_1() {
    mdd(&["fm", "tests/fixtures/no-frontmatter.md"])
        .code(1)
        .stdout("")
        .stderr(no_match(
            "no frontmatter in tests/fixtures/no-frontmatter.md",
        ));
}

#[test]
fn fm_file_with_frontmatter_exits_0() {
    mdd(&["fm", "tests/fixtures/simple.md"])
        .code(0)
        .stdout("topic: testing\ntype: reference\n");
}

// ─── directories without Markdown files ───────────────────────────────

#[test]
fn map_stats_and_fm_on_an_empty_directory_exit_1() {
    let tmp = tempfile::tempdir().unwrap();
    let expected = no_match(&format!("no Markdown files in {}", tmp.path().display()));
    mdd_in(tmp.path(), &["stats"], &[])
        .code(1)
        .stdout("")
        .stderr(expected.clone());
    mdd_in(tmp.path(), &["fm"], &[])
        .code(1)
        .stdout("0/0 files have frontmatter (0%)\n")
        .stderr(expected.clone());
    let dir_name = tmp.path().file_name().unwrap().to_string_lossy();
    mdd_in(tmp.path(), &["map"], &[])
        .code(1)
        .stdout(format!(
            "=== {dir_name}/ (0 files, 0K lines) ===\nTOTAL: 0 files, 0 with tables, 0 with code blocks\n"
        ))
        .stderr(expected);
}

// ─── links ────────────────────────────────────────────────────────────

#[test]
fn links_check_and_broken_exit_1_when_a_local_target_is_broken() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "ok.md", "# Ok\n");
    let path = write(
        tmp.path(),
        "a.md",
        "# A\n\n[ok](ok.md) [bad](nope.md) [ext](https://example.com)\n",
    );
    for flag in ["--check", "--broken"] {
        mdd_in(&path, &["links"], &[flag])
            .code(1)
            .stderr("\n3 links total, 1 broken\n");
    }
    mdd_in(&path, &["links"], &[])
        .code(0)
        .stderr("\n3 links total\n");
}

#[test]
fn links_check_without_a_broken_target_exits_0() {
    mdd(&["links", "tests/fixtures/simple.md", "--check"]).code(0);
}

// ─── errors exit 2 ────────────────────────────────────────────────────

#[test]
fn every_command_exits_2_on_a_missing_path() {
    for args in [
        &["toc", "nonexistent"][..],
        &["sec", "nonexistent", "x"],
        &["at", "nonexistent", "1"],
        &["map", "nonexistent"],
        &["find", "nonexistent"],
        &["fm", "nonexistent"],
        &["links", "nonexistent"],
        &["agg", "nonexistent"],
        &["parts", "nonexistent"],
        &["stats", "nonexistent"],
    ] {
        mdd(args)
            .code(2)
            .stdout("")
            .stderr("mdd: error[file_not_found]: file not found: nonexistent\n");
    }
}
