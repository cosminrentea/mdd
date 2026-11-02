//! Full command-line contract: every subcommand, format and edge input.
//!
//! Each case runs with `-f human`, `-f agent` and `-f json` against copies of
//! `tests/fixtures` and `tests/matrix` placed in a temp directory (relative
//! paths, fixed mtimes, null stdin). One snapshot per subcommand records
//! exit code, stdout and stderr verbatim.
#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

const FORMATS: [&str; 3] = ["human", "agent", "json"];

const SUBCOMMANDS: [&str; 11] = [
    "toc", "sec", "at", "map", "find", "fm", "links", "agg", "parts", "stats", "help",
];

const CASES: &[&str] = &[
    r"--strict find edge12",
    r"--strict toc edge12/badfm.md",
    r"--strict fm edge12",
    r"--strict stats edge12",
    r"find edge12",
    r"toc edge12",
    r"fm edge12",
    r"stats edge12",
    r"parts edge12/badfm.md",
    r"--all-files fm edge",
    r"--all-files find edge -t readme-type",
    r"--all-files toc edge",
    r"map fixtures/simple.md",
    r"stats fixtures/simple.md",
    r"fm edge/plain.md",
    r"parts fixtures/simple.md --filter type=zzz",
    r"parts fixtures/simple.md",
    r"fm edge --list-values nosuchfield",
    r"find fixtures -H nomatchxyz",
    r"links fixtures --check",
    r"map edge12/empty",
    r"stats edge12/empty",
    r"fm edge12/empty",
    r"toc edge/plain.md",
    r"at fixtures/simple.md 1",
    r"sec fixtures/simple.md NOPE_XYZ Simple",
    r"links edge11",
    r"links edge11 --check",
    r"links edge11 --broken",
    r"links edge11/probe.md --check",
    r"links edge11/odd.md --check",
    r"links edge11/places.md --check",
    r"links edge11/multi.md --check",
    r"toc edge11/multi.md",
    r"parts edge11/multi.md",
    r"toc edge11/lists.md",
    r"toc edge11/probe.md",
    r"find edge11",
    r"find edge11 -H lists",
    r"agg edge11 -H flat",
    r"stats edge11",
    r"sec edge11/lists.md Lists",
    r"at edge11/lists.md 4",
    r"fm edge --list-values type",
    r"fm edge/README.md",
    r"toc edge/README.md",
    r"fm edge/AGENTS.md",
    r"links edge --broken",
    r"find edge -t readme-type",
    r"stats edge",
    r"map edge",
    r"-h",
    r"help",
    r"--version",
    r"--bogus",
    r"toc -h",
    r"sec -h",
    r"at -h",
    r"map -h",
    r"find -h",
    r"fm -h",
    r"links -h",
    r"agg -h",
    r"parts -h",
    r"stats -h",
    r"help toc",
    r"help sec",
    r"help at",
    r"help map",
    r"help find",
    r"help fm",
    r"help links",
    r"help agg",
    r"help parts",
    r"help stats",
    r"toc",
    r"sec",
    r"at",
    r"map",
    r"find",
    r"fm",
    r"links",
    r"agg",
    r"parts",
    r"stats",
    r"toc fixtures --bogus",
    r"sec fixtures --bogus",
    r"at fixtures --bogus",
    r"map fixtures --bogus",
    r"find fixtures --bogus",
    r"fm fixtures --bogus",
    r"links fixtures --bogus",
    r"agg fixtures --bogus",
    r"parts fixtures --bogus",
    r"stats fixtures --bogus",
    r"at fixtures/simple.md xyz",
    r"find fixtures --newer-than",
    r"agg fixtures --budget x",
    r"find edge -t REFERENCE",
    r"find edge -t étude",
    r"find edge -t ÉTUDE",
    r"find edge --topic über",
    r"find edge --topic ÜBER",
    r"find edge --topic unit",
    r"find edge -H größe",
    r"find edge -H GRÖSSE",
    r"find edge -H über",
    "find edge --has-code \"\"",
    r"find edge --has-code rust",
    r"find edge --has-code RUST",
    r"find edge --has-table",
    "find fixtures --has-code \"\"",
    r"find fixtures -H QUEUE",
    r"find fixtures -t reference --topic testing --has-table",
    "find edge -t reference -H no --has-code \"\"",
    r"agg edge -H über",
    r"agg edge --topic über",
    r"agg edge -t ÉTUDE",
    r"agg edge -H TABLE",
    r"agg fixtures --topic testing",
    r"agg fixtures -t reference -H a --topic testing",
    r"agg fixtures --budget 5 -H a",
    r"toc fixtures",
    r"toc fixtures/service-doc.md",
    r"toc fixtures/service-doc.md Queue",
    r"toc fixtures/multipart-lrn.md",
    r"toc edge",
    r"toc edge/plain.md",
    r"parts edge/yaml.md",
    r"parts edge/dup.md",
    r"parts edge/list.md",
    r"parts edge/empty.md",
    r"parts edge/keys.md",
    r"parts edge/bad.md",
    r"fm edge/yaml.md",
    r"fm edge/bad.md",
    r"fm edge/dup.md",
    r"fm edge/list.md",
    r"fm edge/empty.md",
    r"fm edge/keys.md",
    r"fm edge",
    r"fm edge --list-values tags",
    r"fm edge --list-values nested",
    r"fm edge --list-values weight",
    r"fm edge --list-values flag",
    r"find edge",
    r"find edge -t reference",
    r"find edge --topic yaml",
    r"stats edge",
    r"toc edge/yaml.md",
    r"agg edge -t reference",
    r"fm edge/yaml.md --set type=zzz",
    r"fm edge/yaml.md --set newkey=v",
    r"fm edge/dup.md --set k=9",
    "sec fixtures/service-doc.md \"Queue Bindings\"",
    r"sec fixtures/service-doc.md Overview Architecture",
    r"sec fixtures Overview",
    r"at fixtures/service-doc.md 10",
    r"at fixtures/service-doc.md 30",
    r"at fixtures/multipart-lrn.md 22",
    r"at fixtures/service-doc.md 9999",
    r"links fixtures",
    r"links fixtures/links-test.md",
    r"links fixtures --broken",
    r"links edge",
    r"parts fixtures/multipart-lrn.md",
    r"parts fixtures/lrn-two-entries.md",
    r"parts edge/multi.md",
    r"stats fixtures",
    r"map fixtures",
    r"map edge",
    r"find fixtures",
    r"find fixtures -t reference",
    r"find fixtures --topic testing",
    r"find fixtures -H Queue",
    r"find fixtures --has-table",
    r"find fixtures --has-code go",
    r"find fixtures --newer-than 2000-01-01",
    r"find fixtures --newer-than 2100-01-01",
    r"agg fixtures -H Queue",
    r"agg fixtures -t reference",
    r"agg fixtures -H nomatchxyz",
    r"fm fixtures",
    r"fm fixtures/service-doc.md",
    r"fm fixtures --list-values type",
    r"fm fixtures --list-values topic",
    r"fm edge/multi.md --set type=zzz",
    r"fm fixtures --set a=b",
    r"toc nonexistent.md",
    r"sec fixtures/simple.md NOPE_XYZ",
    r"at fixtures/simple.md 0",
    r"find nonexistent",
    r"fm nonexistent",
    r"fm fixtures/no-frontmatter.md --set a=b",
    r"fm fixtures/simple.md --set badassign",
    r"links nonexistent",
    r"stats nonexistent",
    r"map nonexistent",
    r"agg nonexistent -t x",
    r"toc fixtures NOPE_XYZ",
    r"find fixtures --newer-than bad",
    r"parts nonexistent",
    r"--help",
    r"toc --help",
    r"sec --help",
    r"at --help",
    r"map --help",
    r"find --help",
    r"fm --help",
    r"links --help",
    r"agg --help",
    r"parts --help",
    r"stats --help",
];

/// Copy a tree and give every file the same mtime.
fn copy_tree(from: &Path, to: &Path, mtime: SystemTime) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &dest, mtime);
        } else {
            std::fs::copy(entry.path(), &dest).unwrap();
            std::fs::OpenOptions::new()
                .write(true)
                .open(&dest)
                .unwrap()
                .set_modified(mtime)
                .unwrap();
        }
    }
}

/// Split a case like the shell would: whitespace separates and double quotes
/// group, so `--has-code ""` passes an empty string.
fn split_args(case: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut in_arg = false;
    for ch in case.chars() {
        match ch {
            '"' => {
                quoted = !quoted;
                in_arg = true;
            }
            c if c.is_whitespace() && !quoted => {
                if in_arg {
                    args.push(std::mem::take(&mut cur));
                    in_arg = false;
                }
            }
            c => {
                cur.push(c);
                in_arg = true;
            }
        }
    }
    if in_arg {
        args.push(cur);
    }
    args
}

/// The subcommand a case exercises, or `global` for bare flags.
fn group_of(args: &[String]) -> &'static str {
    args.iter()
        .find(|a| !a.starts_with('-'))
        .and_then(|first| SUBCOMMANDS.iter().find(|s| **s == first.as_str()))
        .copied()
        .unwrap_or("global")
}

fn workdir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let mtime = SystemTime::now() - Duration::from_secs(3600);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    copy_tree(&root.join("fixtures"), &tmp.path().join("fixtures"), mtime);
    for name in ["edge", "edge11", "edge12"] {
        copy_tree(
            &root.join("matrix").join(name),
            &tmp.path().join(name),
            mtime,
        );
    }
    // An empty directory cannot be committed.
    std::fs::create_dir_all(tmp.path().join("edge12/empty")).unwrap();
    tmp
}

fn render(group: &str) -> String {
    let tmp = workdir();
    let mut blocks = Vec::new();
    for case in CASES {
        let case_args = split_args(case);
        if group_of(&case_args) != group {
            continue;
        }
        for fmt in FORMATS {
            let out = Command::new(env!("CARGO_BIN_EXE_mdd"))
                .current_dir(tmp.path())
                .args(["-f", fmt])
                .args(&case_args)
                .stdin(Stdio::null())
                .output()
                .unwrap();
            let mut block = String::new();
            writeln!(block, "$ mdd -f {fmt} {case}").unwrap();
            writeln!(block, "exit: {}", out.status.code().unwrap_or(-1)).unwrap();
            writeln!(block, "--- stdout").unwrap();
            block.push_str(&String::from_utf8_lossy(&out.stdout));
            writeln!(block, "--- stderr").unwrap();
            block.push_str(&String::from_utf8_lossy(&out.stderr));
            blocks.push(block);
        }
    }
    assert!(!blocks.is_empty(), "no cases for group {group}");
    blocks.join("\n")
}

macro_rules! matrix_tests {
    ($($name:ident),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                insta::assert_snapshot!(stringify!($name), render(stringify!($name)));
            }
        )+
    };
}

matrix_tests!(
    toc, sec, at, map, find, fm, links, agg, parts, stats, help, global
);
