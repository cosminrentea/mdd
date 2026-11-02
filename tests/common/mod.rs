#![allow(dead_code)]

use proptest::prelude::*;
use std::fmt::Write;

// ─── Strategies: generate random markdown ───────────────────────────────────

pub fn arb_heading_level() -> impl Strategy<Value = usize> {
    1..=6usize
}

pub fn arb_word() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-z]{2,12}".prop_map(|s| s)
}

pub fn arb_sentence() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_word(), 3..10).prop_map(|words| words.join(" "))
}

pub fn arb_heading() -> impl Strategy<Value = String> {
    (arb_heading_level(), arb_sentence())
        .prop_map(|(level, text)| format!("{} {}", "#".repeat(level), text))
}

pub fn arb_code_block() -> impl Strategy<Value = String> {
    let langs = prop::sample::select(vec!["rust", "java", "python", "yaml", "go", ""]);
    (langs, arb_sentence()).prop_map(|(lang, content)| format!("```{lang}\n{content}\n```"))
}

pub fn arb_list_item() -> impl Strategy<Value = String> {
    arb_sentence().prop_map(|s| format!("- {s}"))
}

pub fn arb_list() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_list_item(), 1..6).prop_map(|items| items.join("\n"))
}

pub fn arb_table() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_word(), 2..5).prop_map(|cols| {
        let header = format!("| {} |", cols.join(" | "));
        let sep = format!(
            "| {} |",
            cols.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")
        );
        let row = format!(
            "| {} |",
            cols.iter()
                .map(|c| format!("val-{c}"))
                .collect::<Vec<_>>()
                .join(" | ")
        );
        format!("{header}\n{sep}\n{row}")
    })
}

pub fn arb_paragraph() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_sentence(), 1..4).prop_map(|sentences| sentences.join(". ") + ".")
}

pub fn arb_frontmatter() -> impl Strategy<Value = String> {
    let types = prop::sample::select(vec!["reference", "feedback", "note", "project", "user"]);
    let topics = prop::sample::select(vec![
        "testing",
        "deployment",
        "performance",
        "architecture",
        "config",
    ]);
    (types, topics).prop_map(|(t, topic)| format!("---\ntype: {t}\ntopic: {topic}\n---"))
}

pub fn arb_content_block() -> impl Strategy<Value = String> {
    prop_oneof![arb_paragraph(), arb_code_block(), arb_list(), arb_table(),]
}

pub fn arb_section() -> impl Strategy<Value = String> {
    (
        arb_heading(),
        prop::collection::vec(arb_content_block(), 0..3),
    )
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

pub fn arb_markdown_doc() -> impl Strategy<Value = String> {
    let with_fm = (
        arb_frontmatter(),
        prop::collection::vec(arb_section(), 1..8),
    )
        .prop_map(|(fm, sections)| {
            let mut parts = vec![fm, String::new()];
            parts.extend(sections);
            parts.join("\n")
        });

    let without_fm =
        prop::collection::vec(arb_section(), 1..8).prop_map(|sections| sections.join("\n"));

    prop_oneof![
        6 => with_fm,
        2 => without_fm,
        2 => arb_multi_entry_doc(),
    ]
}

/// Several frontmatter blocks, each followed by its own sections.
pub fn arb_multi_entry_doc() -> impl Strategy<Value = String> {
    let entry = (
        arb_frontmatter(),
        prop::collection::vec(arb_section(), 1..4),
    )
        .prop_map(|(fm, sections)| {
            let mut parts = vec![fm, String::new()];
            parts.extend(sections);
            parts.join("\n")
        });
    (
        prop::option::of(arb_sentence()),
        prop::collection::vec(entry, 2..5),
    )
        .prop_map(|(title, entries)| {
            let head = title.map(|t| format!("# {t}\n\n")).unwrap_or_default();
            head + &entries.join("\n")
        })
}

/// A flat run of headings whose levels are unconstrained, so a deeper
/// heading can precede a shallower one.
pub fn arb_level_drop_doc() -> impl Strategy<Value = String> {
    prop::collection::vec((arb_heading_level(), arb_sentence()), 2..8).prop_map(|headings| {
        headings
            .into_iter()
            .fold(String::new(), |mut acc, (level, text)| {
                let _ = write!(acc, "{} {}\n\nbody text\n\n", "#".repeat(level), text);
                acc
            })
    })
}
