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
        .args([
            "-f",
            "human",
            "sec",
            "tests/fixtures/simple.md",
            "Section Two",
        ])
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
        .args([
            "-f",
            "agent",
            "sec",
            "tests/fixtures/simple.md",
            "Section Two",
        ])
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
        .args([
            "-f",
            "agent",
            "agg",
            "tests/fixtures/",
            "-H",
            "Section",
            "--budget",
            "5",
        ])
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
        .stdout(predicate::str::contains("Markdown"));
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
        .args([
            "-f",
            "agent",
            "parts",
            "tests/fixtures/lrn-two-entries.md",
            "--filter",
            "type=note",
        ])
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

// ─── toc: realistic fixtures ──────────────────────────────────────────

#[test]
fn toc_service_doc_shows_nested_headings() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "toc", "tests/fixtures/service-doc.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Nexus Ingestion Pipeline"))
        .stdout(predicate::str::contains("## Architecture"))
        .stdout(predicate::str::contains("### Queue Bindings"))
        .stdout(predicate::str::contains("## Troubleshooting"));
}

#[test]
fn toc_pattern_filters_nested() {
    // "Queue" matches both "### Queue Bindings" and "### High Queue Backlog"
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "toc",
            "tests/fixtures/service-doc.md",
            "Queue",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("### Queue Bindings"))
        .stdout(predicate::str::contains("### High Queue Backlog"))
        .stdout(predicate::str::contains("Document Collections").not());
}

#[test]
fn toc_json_has_all_fields() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "toc", "tests/fixtures/service-doc.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"level\""))
        .stdout(predicate::str::contains("\"title\""))
        .stdout(predicate::str::contains("\"offset\""))
        .stdout(predicate::str::contains("\"limit\""));
}

// ─── sec: realistic fixtures ──────────────────────────────────────────

#[test]
fn sec_extracts_nested_section_with_code() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "human",
            "sec",
            "tests/fixtures/service-doc.md",
            "Environment Variables",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("### Environment Variables"))
        .stdout(predicate::str::contains("STORE_URI"))
        .stdout(predicate::str::contains("```yaml"));
}

#[test]
fn sec_extracts_section_with_table() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "sec",
            "tests/fixtures/service-doc.md",
            "Queue Bindings",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("entity-alpha-p1"))
        .stdout(predicate::str::contains("[tests/fixtures/service-doc.md:"));
}

#[test]
fn sec_case_insensitive_match() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["sec", "tests/fixtures/service-doc.md", "store timeout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store Timeout"));
}

// ─── at: realistic fixtures ───────────────────────────────────────────

#[test]
fn at_finds_section_in_service_doc() {
    // Line 7 is inside the "# Nexus Ingestion Pipeline" section (after frontmatter)
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "at", "tests/fixtures/service-doc.md", "7"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Nexus Ingestion Pipeline"));
}

#[test]
fn at_json_format() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "at", "tests/fixtures/service-doc.md", "15"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"title\""))
        .stdout(predicate::str::contains("\"offset\""));
}

// ─── find: realistic fixtures ─────────────────────────────────────────

#[test]
fn find_filters_by_topic() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "find",
            "tests/fixtures/",
            "--topic",
            "ingestion",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("service-doc.md"))
        .stdout(predicate::str::contains("feedback-entry.md").not());
}

#[test]
fn find_filters_by_code_language() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "find",
            "tests/fixtures/",
            "--has-code",
            "java",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("feedback-entry.md"));
}

#[test]
fn find_type_feedback() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "find", "tests/fixtures/", "-t", "feedback"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feedback-entry.md"))
        .stdout(predicate::str::contains("service-doc.md").not());
}

#[test]
fn find_json_output() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "find", "tests/fixtures/", "-t", "reference"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"path\""))
        .stdout(predicate::str::contains("\"total_lines\""))
        .stdout(predicate::str::contains("\"byte_size\""));
}

#[test]
fn find_nonexistent_dir_fails() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["find", "tests/fixtures/nonexistent-dir/"])
        .assert()
        .failure();
}

// ─── map: realistic fixtures ──────────────────────────────────────────

#[test]
fn map_lists_directory_files() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["map", "tests/fixtures/"])
        .assert()
        .success()
        .stdout(predicate::str::contains("service-doc.md"))
        .stdout(predicate::str::contains("feedback-entry.md"))
        .stdout(predicate::str::contains("TOTAL:"));
}

// ─── agg: realistic fixtures ──────────────────────────────────────────

#[test]
fn agg_type_filter() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "agg",
            "tests/fixtures/",
            "-t",
            "feedback",
            "-H",
            "Good",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("### Good"))
        .stdout(predicate::str::contains("@IntegrationTest"));
}

#[test]
fn agg_emits_content_with_provenance() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "agg",
            "tests/fixtures/",
            "-H",
            "Architecture",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("[service-doc.md:"))
        .stdout(predicate::str::contains("## Architecture"))
        .stdout(predicate::str::contains("Queue"));
}

#[test]
fn agg_json_includes_content() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "json",
            "agg",
            "tests/fixtures/",
            "-H",
            "Troubleshooting",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"content\""))
        .stdout(predicate::str::contains("\"path\""))
        .stdout(predicate::str::contains("health/ready"));
}

#[test]
fn agg_budget_zero_emits_at_least_one() {
    // Budget 0 should still emit at least one section (budget check is "if over AND emitted > 0")
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "agg",
            "tests/fixtures/",
            "-H",
            "Architecture",
            "--budget",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("##"))
        .stderr(predicate::str::contains("1 sections emitted"));
}

// ─── links: realistic fixtures ────────────────────────────────────────

#[test]
fn links_resolves_relative_paths() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", "tests/fixtures/cross-refs.md", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("./service-doc.md (Markdown, OK)"))
        .stdout(predicate::str::contains(
            "./feedback-entry.md (Markdown, OK)",
        ))
        .stdout(predicate::str::contains(
            "./removed-service.md (Markdown, BROKEN)",
        ));
}

#[test]
fn links_external_not_checked() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", "tests/fixtures/cross-refs.md", "--check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://wiki.example.com"))
        .stdout(predicate::str::contains("External"));
}

#[test]
fn links_broken_only_shows_broken() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", "tests/fixtures/cross-refs.md", "--broken"])
        .assert()
        .success()
        .stdout(predicate::str::contains("BROKEN"))
        // Verify no OK or External lines (check for ", OK)" and ", External)" to avoid
        // matching "BROKEN" which contains "OK" as substring)
        .stdout(predicate::str::contains(", OK)").not())
        .stdout(predicate::str::contains("External)").not());
}

#[test]
fn links_directory_mode() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["links", "tests/fixtures/", "--check"])
        .assert()
        .success()
        .stderr(predicate::str::contains("links total"));
}

#[test]
fn links_json_format() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "links", "tests/fixtures/cross-refs.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\""))
        .stdout(predicate::str::contains("\"target\""))
        .stdout(predicate::str::contains("\"line\""));
}

// ─── parts: realistic (multi-entry lrn) ───────────────────────────────

#[test]
fn parts_three_entries_lrn() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "parts", "tests/fixtures/multipart-lrn.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Entry 1"))
        .stdout(predicate::str::contains("Entry 2"))
        .stdout(predicate::str::contains("Entry 3"))
        .stderr(predicate::str::contains("3 entries shown"));
}

#[test]
fn parts_filter_topic() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "parts",
            "tests/fixtures/multipart-lrn.md",
            "--filter",
            "topic=message broker configuration",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Consumer Library"))
        .stdout(predicate::str::contains("HTTP Client").not());
}

#[test]
fn parts_filter_level() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "parts",
            "tests/fixtures/multipart-lrn.md",
            "--filter",
            "level=team",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Entry 3"))
        .stdout(predicate::str::contains("Entry 1").not());
}

#[test]
fn parts_json_format() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "json", "parts", "tests/fixtures/multipart-lrn.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"index\""))
        .stdout(predicate::str::contains("\"offset\""))
        .stdout(predicate::str::contains("\"headings\""));
}

#[test]
fn parts_no_filter_match() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "parts",
            "tests/fixtures/multipart-lrn.md",
            "--filter",
            "type=nonexistent",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("0 entries shown"));
}

// ─── stats: realistic fixtures ────────────────────────────────────────

#[test]
fn stats_type_distribution() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["stats", "tests/fixtures/"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reference("))
        .stdout(predicate::str::contains("feedback("));
}

#[test]
fn stats_content_features() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["stats", "tests/fixtures/"])
        .assert()
        .success()
        .stdout(predicate::str::contains("with tables"))
        .stdout(predicate::str::contains("with code"))
        .stdout(predicate::str::contains("with lists"));
}

#[test]
fn stats_heading_metrics() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["stats", "tests/fixtures/"])
        .assert()
        .success()
        .stdout(predicate::str::contains("avg"))
        .stdout(predicate::str::contains("max depth"));
}

#[test]
fn stats_nonexistent_dir_fails() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["stats", "tests/fixtures/nonexistent-dir/"])
        .assert()
        .failure();
}

// ─── edge cases ─────────────────────────────────────────────────────────────

#[test]
fn toc_file_without_frontmatter() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "toc", "tests/fixtures/no-frontmatter.md"])
        .assert()
        .success();
}

#[test]
fn sec_partial_match_finds_section() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["sec", "tests/fixtures/service-doc.md", "Queue"])
        .assert()
        .success()
        .stdout(predicate::str::contains("High Queue Backlog"));
}

#[test]
fn find_has_table_finds_service_doc() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args(["-f", "agent", "find", "tests/fixtures/", "--has-table"])
        .assert()
        .success()
        .stdout(predicate::str::contains("service-doc.md"));
}

#[test]
fn find_has_code_yaml() {
    Command::cargo_bin("mdd")
        .unwrap()
        .args([
            "-f",
            "agent",
            "find",
            "tests/fixtures/",
            "--has-code",
            "yaml",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("service-doc.md"));
}
