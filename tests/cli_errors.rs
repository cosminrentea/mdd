#![allow(clippy::unwrap_used)]

use assert_cmd::Command;

fn mdd(args: &[&str]) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mdd").unwrap().args(args).assert()
}

// ─── plain error output (exit code 2): `mdd: error[<code>]: <message>` + `  caused by:` lines ───

#[test]
fn missing_file_prints_a_plain_coded_error() {
    mdd(&["toc", "tests/fixtures/nonexistent.md"])
        .code(2)
        .stdout("")
        .stderr("mdd: error[file_not_found]: file not found: tests/fixtures/nonexistent.md\n");
}

#[test]
fn read_failure_prints_the_os_error_as_a_cause() {
    mdd(&["sec", "tests/fixtures", "x"])
        .code(2)
        .stdout("")
        .stderr(
            "mdd: error[io]: failed to read file: tests/fixtures\n  caused by: Is a directory (os error 21)\n",
        );
}

#[test]
fn line_out_of_range_prints_a_plain_coded_error() {
    mdd(&["at", "tests/fixtures/simple.md", "9999"])
        .code(2)
        .stdout("")
        .stderr(predicates::str::starts_with(
            "mdd: error[line_out_of_range]: line 9999 is out of range (file has ",
        ))
        .stderr(predicates::str::ends_with(" lines)\n"));
}

#[test]
fn errors_use_no_colors_glyphs_or_blank_lines() {
    let assert = mdd(&["toc", "tests/fixtures/nonexistent.md"]).code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(stderr.is_ascii(), "stderr: {stderr:?}");
    assert!(!stderr.contains('\u{1b}'), "stderr: {stderr:?}");
    assert!(!stderr.contains("\n\n"), "stderr: {stderr:?}");
    assert!(!stderr.starts_with('\n'), "stderr: {stderr:?}");
}

#[test]
fn map_and_stats_on_a_file_are_not_a_directory_errors() {
    for sub in ["map", "stats"] {
        mdd(&[sub, "tests/fixtures/simple.md"])
            .code(2)
            .stdout("")
            .stderr("mdd: error[not_a_directory]: not a directory: tests/fixtures/simple.md\n");
    }
}

#[test]
fn usage_errors_exit_2() {
    mdd(&["at", "tests/fixtures/simple.md", "xyz"]).code(2);
    mdd(&["--bogus"]).code(2);
}
