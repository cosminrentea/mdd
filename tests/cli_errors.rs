#![allow(clippy::unwrap_used)]

use assert_cmd::Command;

fn mdd(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mdd").unwrap().args(args).assert()
}

// ─── plain error output: `mdd: error[<code>]: <message>` + `  caused by:` lines ───

#[test]
fn missing_file_prints_a_plain_coded_error() {
    mdd(&["toc", "tests/fixtures/nonexistent.md"])
        .code(1)
        .stdout("")
        .stderr("mdd: error[file_not_found]: file not found: tests/fixtures/nonexistent.md\n");
}

#[test]
fn read_failure_prints_the_os_error_as_a_cause() {
    mdd(&["sec", "tests/fixtures", "x"])
        .code(1)
        .stdout("")
        .stderr(
            "mdd: error[io]: failed to read file: tests/fixtures\n  caused by: Is a directory (os error 21)\n",
        );
}

#[test]
fn line_out_of_range_prints_a_plain_coded_error() {
    mdd(&["at", "tests/fixtures/simple.md", "9999"])
        .code(1)
        .stdout("")
        .stderr(predicates::str::starts_with(
            "mdd: error[line_out_of_range]: line 9999 is out of range (file has ",
        ))
        .stderr(predicates::str::ends_with(" lines)\n"));
}

#[test]
fn no_matching_section_prints_a_plain_coded_error() {
    mdd(&["sec", "tests/fixtures/simple.md", "NOPE_XYZ", "NOPE_ABC"])
        .code(1)
        .stdout("")
        .stderr("mdd: error[no_match]: no matching sections for pattern: NOPE_XYZ, NOPE_ABC\n");
}

#[test]
fn file_without_headings_prints_a_plain_coded_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("plain.md");
    std::fs::write(&path, "just text\n").unwrap();
    mdd(&["toc", path.to_str().unwrap()])
        .code(1)
        .stdout("")
        .stderr(format!(
            "mdd: error[no_headings]: no headings found in {}\n",
            path.display()
        ));
}

#[test]
fn errors_use_no_colors_glyphs_or_blank_lines() {
    let assert = mdd(&["toc", "tests/fixtures/nonexistent.md"]).code(1);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.is_ascii(), "stderr: {stderr:?}");
    assert!(!stderr.contains('\u{1b}'), "stderr: {stderr:?}");
    assert!(!stderr.contains("\n\n"), "stderr: {stderr:?}");
    assert!(!stderr.starts_with('\n'), "stderr: {stderr:?}");
}
