#![allow(clippy::unwrap_used)]

use assert_cmd::Command;
use predicates::prelude::*;

// Directory scans skip AGENTS.md, CLAUDE.md, CLAUDE.local.md and README.md
// unless --all-files is given.

const INFRA: [&str; 4] = ["README.md", "AGENTS.md", "CLAUDE.md", "CLAUDE.local.md"];

fn mdd(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mdd").unwrap().args(args).assert()
}

/// `note.md` plus the four infrastructure files; every file links to `target.md`.
fn dir_with_infrastructure_files() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("target.md"), "# Target\n").unwrap();
    std::fs::write(
        tmp.path().join("note.md"),
        "---\ntype: note-type\n---\n\n# Note\n\n[t](target.md)\n",
    )
    .unwrap();
    for name in INFRA {
        std::fs::write(
            tmp.path().join(name),
            "---\ntype: infra-type\n---\n\n# Infra\n\n[t](target.md)\n",
        )
        .unwrap();
    }
    tmp
}

fn path(dir: &tempfile::TempDir) -> &str {
    dir.path().to_str().unwrap()
}

#[test]
fn directory_commands_skip_infrastructure_files_by_default() {
    let tmp = dir_with_infrastructure_files();
    let dir = path(&tmp);
    mdd(&["fm", dir])
        .code(0)
        .stdout("1/2 files have frontmatter (50%)\n");
    mdd(&["stats", dir])
        .code(0)
        .stdout(predicate::str::contains("Files: 2 ("));
    mdd(&["map", dir])
        .code(0)
        .stdout(predicate::str::contains("TOTAL: 2 files"));
    mdd(&["-f", "agent", "find", dir, "-t", "note-type"])
        .code(0)
        .stderr(predicate::str::ends_with("1 files matched\n"));
    mdd(&["-f", "agent", "toc", dir])
        .code(0)
        .stderr(predicate::str::ends_with("2 files\n"));
    mdd(&["agg", dir, "-t", "infra-type"]).code(1);
    mdd(&["links", dir])
        .code(0)
        .stderr(predicate::str::ends_with("1 links total\n"));
    mdd(&["fm", dir, "--list-values", "type"])
        .code(0)
        .stdout("note-type\n");
}

#[test]
fn all_files_includes_the_infrastructure_files_in_every_directory_command() {
    let tmp = dir_with_infrastructure_files();
    let dir = path(&tmp);
    mdd(&["--all-files", "fm", dir])
        .code(0)
        .stdout("5/6 files have frontmatter (83%)\n");
    mdd(&["--all-files", "stats", dir])
        .code(0)
        .stdout(predicate::str::contains("Files: 6 ("));
    mdd(&["--all-files", "map", dir])
        .code(0)
        .stdout(predicate::str::contains("TOTAL: 6 files"));
    mdd(&[
        "--all-files",
        "-f",
        "agent",
        "find",
        dir,
        "-t",
        "infra-type",
    ])
    .code(0)
    .stderr(predicate::str::ends_with("4 files matched\n"));
    mdd(&["--all-files", "-f", "agent", "toc", dir])
        .code(0)
        .stderr(predicate::str::ends_with("6 files\n"));
    mdd(&["--all-files", "agg", dir, "-t", "infra-type"])
        .code(0)
        .stderr(predicate::str::contains("4 sections emitted"));
    mdd(&["--all-files", "links", dir])
        .code(0)
        .stderr(predicate::str::ends_with("5 links total\n"));
    mdd(&["--all-files", "fm", dir, "--list-values", "type"])
        .code(0)
        .stdout("infra-type\nnote-type\n");
}

#[test]
fn all_files_is_global_and_works_after_the_subcommand() {
    let tmp = dir_with_infrastructure_files();
    mdd(&["stats", path(&tmp), "--all-files"])
        .code(0)
        .stdout(predicate::str::contains("Files: 6 ("));
}

#[test]
fn only_the_exact_infrastructure_names_are_skipped() {
    let tmp = dir_with_infrastructure_files();
    std::fs::write(tmp.path().join("MY-README.md"), "# prefixed\n").unwrap();
    std::fs::write(tmp.path().join("README.md.md"), "# double\n").unwrap();
    mdd(&["stats", path(&tmp)])
        .code(0)
        .stdout(predicate::str::contains("Files: 4 ("));
}

#[test]
fn a_directory_with_only_infrastructure_files_is_empty_without_the_flag() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("README.md"), "# Readme\n").unwrap();
    mdd(&["stats", path(&tmp)]).code(1);
    mdd(&["--all-files", "stats", path(&tmp)]).code(0);
}

#[test]
fn a_file_named_on_the_command_line_is_read_either_way() {
    let tmp = dir_with_infrastructure_files();
    let readme = tmp.path().join("README.md");
    let readme = readme.to_str().unwrap();
    mdd(&["fm", readme]).code(0).stdout("type: infra-type\n");
    mdd(&["--all-files", "fm", readme])
        .code(0)
        .stdout("type: infra-type\n");
}

#[test]
fn every_help_screen_documents_the_option() {
    let help = |args: &[&str]| {
        let assert = mdd(args).code(0);
        String::from_utf8(assert.get_output().stdout.clone()).unwrap()
    };
    let description =
        "Include AGENTS.md, CLAUDE.md, CLAUDE.local.md and README.md in directory scans";
    for sub in [
        "toc", "sec", "at", "map", "find", "fm", "links", "agg", "parts", "stats",
    ] {
        for flag in ["--help", "-h"] {
            let text = help(&[sub, flag]);
            assert!(text.contains("--all-files"), "{sub} {flag}: {text}");
            assert!(text.contains(description), "{sub} {flag}: {text}");
        }
    }
    assert!(help(&["--help"]).contains("--all-files"));
    let short = help(&["-h"]);
    assert!(
        short
            .lines()
            .any(|l| l.trim_start().starts_with("--all-files ") && l.ends_with(description)),
        "{short}"
    );
}
