#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

fn fm_set(path: &Path, assignment: &str, apply: bool) -> assert_cmd::assert::Assert {
    let mut cmd = Command::cargo_bin("mdd").unwrap();
    cmd.arg("fm").arg(path).args(["--set", assignment]);
    if apply {
        cmd.arg("--apply");
    }
    cmd.assert()
}

fn write(dir: &Path, name: &str, content: &[u8]) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap()
}

fn mdd_fixture_copy(dir: &Path) -> (std::path::PathBuf, String) {
    let original = std::fs::read_to_string("tests/fixtures/multipart-lrn.md").unwrap();
    (write(dir, "multi.md", original.as_bytes()), original)
}

// ─── characterization of legitimate blocks (unchanged behavior) ───────

#[test]
fn apply_rewrites_the_key_in_every_entry_block() {
    let tmp = tempfile::tempdir().unwrap();
    let (path, original) = mdd_fixture_copy(tmp.path());
    fm_set(&path, "type=new", true).success();
    let expected = original
        .replace("type: reference", "type: new")
        .replace("type: note", "type: new");
    assert_eq!(String::from_utf8(read(&path)).unwrap(), expected);
}

#[test]
fn apply_appends_an_absent_key_to_the_first_block_only() {
    let tmp = tempfile::tempdir().unwrap();
    let (path, original) = mdd_fixture_copy(tmp.path());
    fm_set(&path, "extra=v", true).success();
    let mut lines: Vec<&str> = original.split_inclusive('\n').collect();
    lines.insert(9, "extra: v\n");
    assert_eq!(String::from_utf8(read(&path)).unwrap(), lines.concat());
}

#[test]
fn apply_replaces_in_later_blocks_and_appends_to_the_first() {
    let tmp = tempfile::tempdir().unwrap();
    let doc =
        "---\ntype: a\n---\n\n# One\n\ntext\n\n---\ntype: b\nextra: old\n---\n\n# Two\n\ntext\n";
    let path = write(tmp.path(), "two.md", doc.as_bytes());
    fm_set(&path, "extra=v", true).success();
    let expected = "---\ntype: a\nextra: v\n---\n\n# One\n\ntext\n\n---\ntype: b\nextra: v\n---\n\n# Two\n\ntext\n";
    assert_eq!(read(&path), expected.as_bytes());
}

#[test]
fn apply_leaves_nested_and_indented_keys_alone() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = "---\ntype: a\nmeta:\n  type: keep\n---\n\n# H\n";
    let path = write(tmp.path(), "nested.md", doc.as_bytes());
    fm_set(&path, "type=new", true).success();
    assert_eq!(
        read(&path),
        "---\ntype: new\nmeta:\n  type: keep\n---\n\n# H\n".as_bytes()
    );
}

#[test]
fn apply_preserves_a_missing_trailing_newline() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write(tmp.path(), "nonl.md", b"---\ntype: a\n---\n\n# H\n\ntext");
    fm_set(&path, "type=new", true).success();
    assert_eq!(read(&path), b"---\ntype: new\n---\n\n# H\n\ntext");
}

#[test]
fn apply_keeps_bom_when_block_is_not_on_line_one() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = "\u{feff}# T\n\n---\ntype: a\n---\n\n# H\n";
    let path = write(tmp.path(), "bom.md", doc.as_bytes());
    fm_set(&path, "type=new", true).success();
    assert_eq!(
        read(&path),
        "\u{feff}# T\n\n---\ntype: new\n---\n\n# H\n".as_bytes()
    );
}

#[test]
fn bom_before_first_delimiter_means_no_frontmatter_error() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = "\u{feff}---\ntype: a\n---\n\n# H\n";
    let path = write(tmp.path(), "bom1.md", doc.as_bytes());
    fm_set(&path, "type=new", true)
        .code(1)
        .stderr(predicate::str::contains("no frontmatter"));
    assert_eq!(read(&path), doc.as_bytes());
}

#[test]
fn dry_run_leaves_the_file_byte_identical() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = b"---\ntype: a\n---\n\n# H\n";
    let path = write(tmp.path(), "dry.md", doc);
    fm_set(&path, "type=new", false)
        .success()
        .stdout(predicate::str::contains("type: new"));
    assert_eq!(read(&path), doc);
}

#[test]
fn apply_changes_only_the_frontmatter() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write(
        tmp.path(),
        "simple.md",
        b"---\ntype: a\ntopic: t\n---\n\n# H\n\nbody\n",
    );
    fm_set(&path, "type=new", true).success();
    assert_eq!(
        read(&path),
        b"---\ntype: new\ntopic: t\n---\n\n# H\n\nbody\n"
    );
}

// ─── fixes ────────────────────────────────────────────────────────────

#[test]
fn body_thematic_break_followed_by_key_is_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = "---\ntype: a\n---\n\n# H\n\ntext\n\n---\ntype: body-line\n\nmore\n";
    let path = write(tmp.path(), "body.md", doc.as_bytes());
    fm_set(&path, "type=new", true).success();
    let expected = doc.replacen("type: a", "type: new", 1);
    assert_eq!(read(&path), expected.as_bytes());
}

#[test]
fn frontmatter_lookalike_in_a_code_fence_is_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = "---\ntype: a\n---\n\n# H\n\n```yaml\n---\ntype: sample\n---\n```\n";
    let path = write(tmp.path(), "fence.md", doc.as_bytes());
    fm_set(&path, "type=new", true).success();
    let expected = doc.replacen("type: a", "type: new", 1);
    assert_eq!(read(&path), expected.as_bytes());
}

#[test]
fn crlf_line_endings_are_preserved() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = "---\r\ntype: a\r\ntopic: t\r\n---\r\n\r\n# H\r\n\r\ntext\r\n";
    let path = write(tmp.path(), "crlf.md", doc.as_bytes());
    fm_set(&path, "type=new", true).success();
    assert_eq!(
        read(&path),
        "---\r\ntype: new\r\ntopic: t\r\n---\r\n\r\n# H\r\n\r\ntext\r\n".as_bytes()
    );
    fm_set(&path, "extra=v", true).success();
    assert_eq!(
        read(&path),
        "---\r\ntype: new\r\ntopic: t\r\nextra: v\r\n---\r\n\r\n# H\r\n\r\ntext\r\n".as_bytes()
    );
}

#[test]
fn set_on_a_directory_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "a.md", b"---\ntype: a\n---\n\n# H\n");
    fm_set(tmp.path(), "type=new", true)
        .code(1)
        .stderr(predicate::str::contains("single files"));
    assert_eq!(
        read(&tmp.path().join("a.md")),
        b"---\ntype: a\n---\n\n# H\n"
    );
}

#[test]
fn apply_without_change_does_not_rewrite_the_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write(tmp.path(), "same.md", b"---\ntype: a\n---\n\n# H\n");
    let old = UNIX_EPOCH + Duration::from_secs(1_577_836_800);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(old)
        .unwrap();
    fm_set(&path, "type=a", true).success();
    assert_eq!(std::fs::metadata(&path).unwrap().modified().unwrap(), old);
}
