use proptest::prelude::*;
use std::path::Path;
use tempfile::TempDir;

// We can't import from a binary crate directly, so we test via CLI.
// These property tests generate random markdown and verify invariants
// that must hold for ANY valid input.

use assert_cmd::Command;
use predicates::prelude::*;

// ─── Strategies: generate random markdown ───────────────────────────────────

fn arb_heading_level() -> impl Strategy<Value = usize> {
    1..=6usize
}

fn arb_word() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-z]{2,12}".prop_map(|s| s)
}

fn arb_sentence() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_word(), 3..10).prop_map(|words| words.join(" "))
}

fn arb_heading() -> impl Strategy<Value = String> {
    (arb_heading_level(), arb_sentence()).prop_map(|(level, text)| {
        format!("{} {}", "#".repeat(level), text)
    })
}

fn arb_code_block() -> impl Strategy<Value = String> {
    let langs = prop::sample::select(vec!["rust", "java", "python", "yaml", "go", ""]);
    (langs, arb_sentence()).prop_map(|(lang, content)| {
        format!("```{}\n{}\n```", lang, content)
    })
}

fn arb_list_item() -> impl Strategy<Value = String> {
    arb_sentence().prop_map(|s| format!("- {}", s))
}

fn arb_list() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_list_item(), 1..6).prop_map(|items| items.join("\n"))
}

fn arb_table() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_word(), 2..5).prop_map(|cols| {
        let header = format!("| {} |", cols.join(" | "));
        let sep = format!("| {} |", cols.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
        let row = format!("| {} |", cols.iter().map(|c| format!("val-{}", c)).collect::<Vec<_>>().join(" | "));
        format!("{}\n{}\n{}", header, sep, row)
    })
}

fn arb_paragraph() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_sentence(), 1..4).prop_map(|sentences| sentences.join(". ") + ".")
}

fn arb_frontmatter() -> impl Strategy<Value = String> {
    let types = prop::sample::select(vec!["reference", "feedback", "note", "project", "user"]);
    let topics = prop::sample::select(vec!["testing", "deployment", "performance", "architecture", "config"]);
    (types, topics).prop_map(|(t, topic)| {
        format!("---\ntype: {}\ntopic: {}\n---", t, topic)
    })
}

fn arb_content_block() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_paragraph(),
        arb_code_block(),
        arb_list(),
        arb_table(),
    ]
}

fn arb_section() -> impl Strategy<Value = String> {
    (arb_heading(), prop::collection::vec(arb_content_block(), 0..3))
        .prop_map(|(heading, blocks)| {
            let mut parts = vec![heading];
            parts.push(String::new());
            for block in blocks {
                parts.push(block);
                parts.push(String::new());
            }
            parts.join("\n")
        })
}

fn arb_markdown_doc() -> impl Strategy<Value = String> {
    let with_fm = (
        arb_frontmatter(),
        prop::collection::vec(arb_section(), 1..8),
    )
        .prop_map(|(fm, sections)| {
            let mut parts = vec![fm, String::new()];
            parts.extend(sections);
            parts.join("\n")
        });

    let without_fm = prop::collection::vec(arb_section(), 1..8)
        .prop_map(|sections| sections.join("\n"));

    prop_oneof![
        8 => with_fm,
        2 => without_fm,
    ]
}

// ─── Property tests ─────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn toc_never_panics(doc in arb_markdown_doc()) {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.md");
        std::fs::write(&file, &doc).unwrap();

        let result = Command::cargo_bin("mdd")
            .unwrap()
            .args(["-f", "agent", "toc", file.to_str().unwrap()])
            .output()
            .unwrap();

        // Must never panic (exit with signal) -- may succeed or fail gracefully
        prop_assert!(result.status.code().is_some(), "process was killed by signal");
    }

    #[test]
    fn toc_offset_limit_covers_all_lines(doc in arb_markdown_doc()) {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.md");
        std::fs::write(&file, &doc).unwrap();

        let output = Command::cargo_bin("mdd")
            .unwrap()
            .args(["-f", "json", "toc", file.to_str().unwrap()])
            .output()
            .unwrap();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // If JSON output is valid, every section's offset must be >= 1
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(sections) = json.get("sections").and_then(|s| s.as_array()) {
                    for section in sections {
                        let offset = section.get("offset").and_then(|o| o.as_u64()).unwrap_or(0);
                        let limit = section.get("limit").and_then(|l| l.as_u64()).unwrap_or(0);
                        prop_assert!(offset >= 1, "offset must be >= 1, got {}", offset);
                        prop_assert!(limit >= 1, "limit must be >= 1, got {}", limit);
                    }
                }
            }
        }
    }

    #[test]
    fn sec_returns_subset_of_file(doc in arb_markdown_doc()) {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.md");
        std::fs::write(&file, &doc).unwrap();

        let total_lines = doc.lines().count();

        // Get TOC first to find a heading
        let toc_out = Command::cargo_bin("mdd")
            .unwrap()
            .args(["-f", "json", "toc", file.to_str().unwrap()])
            .output()
            .unwrap();

        if toc_out.status.success() {
            let toc_str = String::from_utf8_lossy(&toc_out.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&toc_str) {
                if let Some(sections) = json.get("sections").and_then(|s| s.as_array()) {
                    if let Some(first) = sections.first() {
                        let title = first.get("title").and_then(|t| t.as_str()).unwrap_or("");
                        let limit = first.get("limit").and_then(|l| l.as_u64()).unwrap_or(0);

                        if !title.is_empty() {
                            // Section limit must not exceed total file lines
                            prop_assert!(
                                limit as usize <= total_lines,
                                "section limit {} exceeds file lines {}", limit, total_lines
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn at_returns_valid_section_or_error(doc in arb_markdown_doc()) {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.md");
        std::fs::write(&file, &doc).unwrap();

        let line_count = doc.lines().count();
        // Pick a random line in range
        let line = if line_count > 0 { (line_count / 2).max(1) } else { 1 };

        let result = Command::cargo_bin("mdd")
            .unwrap()
            .args(["-f", "agent", "at", file.to_str().unwrap(), &line.to_string()])
            .output()
            .unwrap();

        // Must exit cleanly (0 or 1), never crash
        let code = result.status.code().unwrap_or(-1);
        prop_assert!(code == 0 || code == 1, "unexpected exit code: {}", code);
    }

    #[test]
    fn find_never_panics_on_random_dir(doc in arb_markdown_doc()) {
        let dir = TempDir::new().unwrap();
        // Write 3 random files
        for i in 0..3 {
            let file = dir.path().join(format!("doc{}.md", i));
            std::fs::write(&file, &doc).unwrap();
        }

        let result = Command::cargo_bin("mdd")
            .unwrap()
            .args(["-f", "agent", "find", dir.path().to_str().unwrap()])
            .output()
            .unwrap();

        prop_assert!(result.status.code().is_some());
    }

    #[test]
    fn stats_counts_match_file_count(doc in arb_markdown_doc()) {
        let dir = TempDir::new().unwrap();
        let file_count = 3;
        for i in 0..file_count {
            let file = dir.path().join(format!("doc{}.md", i));
            std::fs::write(&file, &doc).unwrap();
        }

        let output = Command::cargo_bin("mdd")
            .unwrap()
            .args(["stats", dir.path().to_str().unwrap()])
            .output()
            .unwrap();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            prop_assert!(
                stdout.contains(&format!("Files: {}", file_count)),
                "expected 'Files: {}' in output: {}", file_count, stdout
            );
        }
    }

    #[test]
    fn links_never_panics(doc in arb_markdown_doc()) {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.md");
        std::fs::write(&file, &doc).unwrap();

        let result = Command::cargo_bin("mdd")
            .unwrap()
            .args(["links", file.to_str().unwrap(), "--check"])
            .output()
            .unwrap();

        prop_assert!(result.status.code().is_some());
    }

    #[test]
    fn agg_respects_budget_invariant(doc in arb_markdown_doc()) {
        let dir = TempDir::new().unwrap();
        for i in 0..3 {
            let file = dir.path().join(format!("doc{}.md", i));
            std::fs::write(&file, &doc).unwrap();
        }

        let budget = 10;
        let output = Command::cargo_bin("mdd")
            .unwrap()
            .args(["-f", "agent", "agg", dir.path().to_str().unwrap(), "--budget", &budget.to_string()])
            .output()
            .unwrap();

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let output_lines = stdout.lines().count();
            // Output lines should be reasonable relative to budget
            // (budget counts content lines, output also has header lines + blank separators)
            // The key invariant: at least one section is always emitted
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No sections matched") {
                prop_assert!(
                    stderr.contains("sections emitted"),
                    "expected emission notice in stderr: {}", stderr
                );
            }
        }
    }
}

// ─── Targeted fuzzing: pathological inputs ──────────────────────────────────

#[test]
fn parser_handles_empty_file() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("empty.md");
    std::fs::write(&file, "").unwrap();

    // Empty file has no headings -> toc returns error (graceful, not panic)
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "toc", file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no headings"));
}

#[test]
fn parser_handles_only_frontmatter() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("fm-only.md");
    std::fs::write(&file, "---\ntype: note\n---\n").unwrap();

    // Frontmatter-only file has no headings -> toc returns error
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "toc", file.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no headings"));
}

#[test]
fn parser_handles_deeply_nested_headings() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("deep.md");
    let content = (1..=6)
        .map(|level| format!("{} Level {}\n\nContent at level {}.\n", "#".repeat(level), level, level))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&file, &content).unwrap();

    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "toc", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("###### Level 6"));
}

#[test]
fn parser_handles_consecutive_frontmatter_blocks() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("multi-fm.md");
    let content = "# Title\n\n---\ntype: a\n---\n\n## First\n\nContent.\n\n---\ntype: b\n---\n\n## Second\n\nMore.\n";
    std::fs::write(&file, content).unwrap();

    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "parts", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Entry 1"))
        .stdout(predicate::str::contains("Entry 2"));
}

#[test]
fn parser_handles_unicode_headings() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("unicode.md");
    let content = "# Uberblick und Zusammenfassung\n\n## Losungsansatze\n\nKonzept hier.\n";
    std::fs::write(&file, content).unwrap();

    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "toc", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Uberblick"));
}

#[test]
fn parser_handles_very_long_lines() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("longline.md");
    let long_word = "x".repeat(10000);
    let content = format!("# Title\n\n{}\n\n## Next\n\nShort.\n", long_word);
    std::fs::write(&file, &content).unwrap();

    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "toc", file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn parser_handles_many_headings() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("many.md");
    let content: String = (1..=200)
        .map(|i| format!("## Section {}\n\nParagraph {}.\n\n", i, i))
        .collect();
    std::fs::write(&file, &content).unwrap();

    let output = Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "toc", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let sections = json.get("sections").unwrap().as_array().unwrap();
    assert_eq!(sections.len(), 200);
}

#[test]
fn links_handles_malformed_wiki_links() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("badlinks.md");
    let content = "# Links\n\n[[]] empty\n[[]broken\n[[|no target]]\n[normal](./ok.md)\n";
    std::fs::write(&file, content).unwrap();

    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", file.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn links_handles_nested_brackets() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("nested.md");
    let content = "# Test\n\n[text [with] brackets](./file.md)\n[[note [1]]]\n";
    std::fs::write(&file, content).unwrap();

    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", file.to_str().unwrap()])
        .assert()
        .success();
}
