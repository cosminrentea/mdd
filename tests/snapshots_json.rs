use assert_cmd::Command;

fn run(args: &[&str]) -> String {
    let output = Command::cargo_bin("mdd")
        .unwrap()
        .args(args)
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn snap_toc_json() {
    insta::assert_snapshot!(run(&["-f", "json", "toc", "tests/fixtures/service-doc.md"]));
}

#[test]
fn snap_toc_agent() {
    insta::assert_snapshot!(run(&[
        "-f",
        "agent",
        "toc",
        "tests/fixtures/service-doc.md"
    ]));
}

#[test]
fn snap_sec_json() {
    insta::assert_snapshot!(run(&[
        "-f",
        "json",
        "sec",
        "tests/fixtures/service-doc.md",
        "Queue"
    ]));
}

#[test]
fn snap_find_json() {
    insta::assert_snapshot!(run(&[
        "-f",
        "json",
        "find",
        "tests/fixtures/",
        "-t",
        "reference"
    ]));
}

#[test]
fn snap_links_json() {
    insta::assert_snapshot!(run(&[
        "-f",
        "json",
        "links",
        "tests/fixtures/links-test.md",
        "--check"
    ]));
}

#[test]
fn snap_parts_json() {
    insta::assert_snapshot!(run(&[
        "-f",
        "json",
        "parts",
        "tests/fixtures/multipart-lrn.md"
    ]));
}

#[test]
fn snap_stats() {
    insta::assert_snapshot!(run(&["stats", "tests/fixtures/"]));
}
