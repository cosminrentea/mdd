#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};

// A problem with one file or one frontmatter block is a warning on stderr and
// processing continues; with --strict the first such problem is an error (exit 2).

fn mdd(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mdd").unwrap().args(args).assert()
}

fn write(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

const BAD: &str = "---\nkey: [unclosed\n---\n# Broken\n";
const LIST: &str = "---\n- a\n---\n# Listed\n";
const GOOD: &str = "---\ntype: ok\n---\n# Fine\n";

/// `^mdd: warning: <path>:<line>: invalid frontmatter ignored: <one-line reason>\n$`
fn warning_line(path: &Path, line: usize) -> impl Predicate<str> {
    let re = format!(
        "^mdd: warning: {}:{line}: invalid frontmatter ignored: [^\\n]+\\n$",
        regex_escape(&path.display().to_string())
    );
    predicates::str::is_match(re).unwrap()
}

fn regex_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_ascii_alphanumeric() || c == '/' || c == '_' || c == '-' {
                vec![c]
            } else {
                vec!['\\', c]
            }
        })
        .collect()
}

fn stderr_of(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8(assert.get_output().stderr.clone()).unwrap()
}

#[test]
fn malformed_yaml_warns_once_and_processing_continues() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write(tmp.path(), "bad.md", BAD.as_bytes());
    let assert = mdd(&["toc", path.to_str().unwrap()]).code(0);
    assert
        .stdout(predicate::str::contains("Broken"))
        .stderr(warning_line(&path, 1));
}

#[test]
fn a_frontmatter_block_that_is_not_a_mapping_warns() {
    let tmp = tempfile::tempdir().unwrap();
    for (name, doc) in [("list.md", LIST), ("scalar.md", "---\nfoo\n---\n# S\n")] {
        let path = write(tmp.path(), name, doc.as_bytes());
        mdd(&["fm", path.to_str().unwrap()])
            .code(1)
            .stderr(predicate::str::contains(format!(
                "mdd: warning: {}:1: invalid frontmatter ignored: ",
                path.display()
            )))
            .stderr(predicate::str::ends_with(format!(
                "mdd: no match: no frontmatter in {}\n",
                path.display()
            )));
    }
}

#[test]
fn the_warning_names_the_line_of_the_malformed_block() {
    let tmp = tempfile::tempdir().unwrap();
    let doc = "---\ntype: ok\n---\n# One\n\n---\nkey: [unclosed\n---\n# Two\n";
    let path = write(tmp.path(), "multi.md", doc.as_bytes());
    mdd(&["parts", path.to_str().unwrap()])
        .code(0)
        .stderr(predicate::str::contains(format!(
            "mdd: warning: {}:6: invalid frontmatter ignored: ",
            path.display()
        )))
        .stderr(predicate::str::ends_with("2 entries shown (of 2 total)\n"));
}

#[test]
fn an_empty_frontmatter_block_stays_silent() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write(tmp.path(), "empty.md", b"---\n---\n# Empty fm\n");
    mdd(&["toc", path.to_str().unwrap()]).code(0).stderr("");
}

#[test]
fn directory_scans_warn_once_per_problem_in_file_order() {
    let tmp = tempfile::tempdir().unwrap();
    let b = write(tmp.path(), "b.md", BAD.as_bytes());
    let a = write(tmp.path(), "a.md", LIST.as_bytes());
    write(tmp.path(), "c.md", GOOD.as_bytes());
    let dir = tmp.path().to_str().unwrap();
    for sub in ["stats", "map", "fm", "toc", "find", "agg", "links"] {
        let assert = mdd(&[sub, dir]);
        let err = stderr_of(&assert);
        let warnings: Vec<&str> = err
            .lines()
            .filter(|l| l.starts_with("mdd: warning: "))
            .collect();
        assert_eq!(warnings.len(), 2, "{sub}: {err}");
        assert!(
            warnings[0].starts_with(&format!(
                "mdd: warning: {}:1: invalid frontmatter ignored: ",
                a.display()
            )),
            "{sub}: {err}"
        );
        assert!(
            warnings[1].starts_with(&format!(
                "mdd: warning: {}:1: invalid frontmatter ignored: ",
                b.display()
            )),
            "{sub}: {err}"
        );
    }
}

#[test]
fn a_warning_does_not_change_the_exit_code() {
    let tmp = tempfile::tempdir().unwrap();
    let bad = write(tmp.path(), "bad.md", BAD.as_bytes());
    write(tmp.path(), "good.md", GOOD.as_bytes());
    let dir = tmp.path().to_str().unwrap();
    mdd(&["find", dir, "-t", "ok"]).code(0);
    let assert = mdd(&["find", dir, "-t", "nomatchxyz"]).code(1);
    let err = stderr_of(&assert);
    assert!(
        err.starts_with(&format!("mdd: warning: {}:1: ", bad.display())),
        "{err}"
    );
    assert!(
        err.ends_with("\nmdd: no match: no files matched the given filters\n"),
        "{err}"
    );
}

#[test]
fn an_unreadable_file_in_a_directory_scan_warns_and_is_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = write(tmp.path(), "bin.md", &[0xff, 0xfe, 0xfd]);
    write(tmp.path(), "good.md", GOOD.as_bytes());
    let dir = tmp.path().to_str().unwrap();
    mdd(&["find", dir])
        .code(0)
        .stdout(predicate::str::contains("Fine"))
        .stderr(predicate::str::contains(format!(
            "mdd: warning: skipping {}: ",
            bin.display()
        )));
}

#[cfg(unix)]
#[test]
fn an_unreadable_directory_in_a_walk_warns_and_is_skipped() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    write(tmp.path(), "good.md", GOOD.as_bytes());
    let locked = tmp.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    write(&locked, "hidden.md", GOOD.as_bytes());
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    let probe = std::fs::read_dir(&locked).is_ok();
    let assert = mdd(&["find", tmp.path().to_str().unwrap()]);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    if probe {
        return; // running with privileges that ignore permissions
    }
    assert.code(0).stderr(predicate::str::contains(format!(
        "mdd: warning: skipping {}: Permission denied",
        locked.display()
    )));
}

// ─── --strict ─────────────────────────────────────────────────────────

#[test]
fn strict_turns_the_first_frontmatter_problem_into_an_error_with_no_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    let bad = write(tmp.path(), "bad.md", BAD.as_bytes());
    write(tmp.path(), "good.md", GOOD.as_bytes());
    let dir = tmp.path().to_str().unwrap();
    for args in [
        vec!["--strict", "toc", bad.to_str().unwrap()],
        vec!["--strict", "sec", bad.to_str().unwrap(), "Broken"],
        vec!["--strict", "parts", bad.to_str().unwrap()],
        vec!["--strict", "find", dir],
        vec!["--strict", "fm", dir],
        vec!["--strict", "stats", dir],
        vec!["--strict", "map", dir],
        vec!["--strict", "agg", dir],
        vec!["--strict", "links", dir],
        vec!["--strict", "toc", dir],
    ] {
        let warned = mdd(&args[1..]);
        let warning = stderr_of(&warned);
        let text = warning
            .lines()
            .find_map(|l| l.strip_prefix("mdd: warning: "))
            .unwrap()
            .to_owned();
        mdd(&args)
            .code(2)
            .stdout("")
            .stderr(format!("mdd: error[invalid_frontmatter]: {text}\n"));
    }
}

#[test]
fn strict_stops_at_the_first_problem_in_file_order() {
    let tmp = tempfile::tempdir().unwrap();
    let a = write(tmp.path(), "a.md", LIST.as_bytes());
    write(tmp.path(), "b.md", BAD.as_bytes());
    let assert = mdd(&["--strict", "stats", tmp.path().to_str().unwrap()]).code(2);
    let err = stderr_of(&assert);
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(
        err.starts_with(&format!(
            "mdd: error[invalid_frontmatter]: {}:1: ",
            a.display()
        )),
        "{err}"
    );
}

#[test]
fn strict_turns_an_unreadable_file_into_an_io_error() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = write(tmp.path(), "bin.md", &[0xff, 0xfe, 0xfd]);
    write(tmp.path(), "good.md", GOOD.as_bytes());
    let dir = tmp.path().to_str().unwrap();
    let warned = mdd(&["find", dir]);
    let text = stderr_of(&warned)
        .lines()
        .find_map(|l| l.strip_prefix("mdd: warning: "))
        .unwrap()
        .to_owned();
    assert!(text.starts_with(&format!("skipping {}: ", bin.display())));
    mdd(&["--strict", "find", dir])
        .code(2)
        .stdout("")
        .stderr(format!("mdd: error[io]: {text}\n"));
}

#[test]
fn strict_is_silent_when_nothing_is_wrong() {
    mdd(&["--strict", "toc", "tests/fixtures/simple.md"])
        .code(0)
        .stderr("");
    mdd(&["--strict", "stats", "tests/fixtures"]).code(0);
}
