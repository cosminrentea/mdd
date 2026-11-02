// Core Markdown parser: converts raw file content into our MdFile data model.
//
// Uses pulldown-cmark's OffsetIter for a single streaming pass through the
// document. The key insight: pulldown-cmark emits (Event, Range<usize>) pairs
// where the Range is the byte offset span in the source string. We convert
// byte offsets to 1-based line numbers using a pre-computed lookup table.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use pulldown_cmark::{
    Event, HeadingLevel, LinkType, MetadataBlockKind, Options, Parser, Tag, TagEnd,
};

use super::features::FeatureAccumulator;
use super::frontmatter::parse_frontmatter;
use super::links::make_link;
use super::{ContentFeatures, DocEntry, Frontmatter, Link, MdFile, Section};
use crate::error::{MddError, Result};

// ─── Public API ───────────────────────────────────────────────────────────────

/// Parse a Markdown file from disk into an `MdFile` struct.
///
/// Reads the file (kept as-is in `MdFile::source`), gets mtime, then parses the text.
pub fn parse_file(path: &Path) -> Result<MdFile> {
    if !path.exists() {
        return Err(MddError::FileNotFound {
            path: path.to_owned(),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| MddError::Io {
        path: path.to_owned(),
        source: e,
    })?;

    let metadata = fs::metadata(path).map_err(|e| MddError::Io {
        path: path.to_owned(),
        source: e,
    })?;

    let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    let byte_size = metadata.len();

    // Links resolve against the file's own directory.
    let file_dir = path.parent().unwrap_or(Path::new("."));
    let mut md_file = parse_owned(content, file_dir);
    path.clone_into(&mut md_file.path);
    md_file.byte_size = byte_size;
    md_file.mtime = mtime;

    Ok(md_file)
}

/// Parse a Markdown string into an `MdFile` struct (no filesystem access).
///
/// This is the workhorse function. It:
/// 1. Builds a `line_starts` lookup table
/// 2. Iterates pulldown-cmark events with byte offsets
/// 3. Detects entry boundaries (`MetadataBlock` events)
/// 4. Tracks headings and their hierarchical relationships
/// 5. Accumulates `ContentFeatures` per section
/// 6. Collects the links (resolved against an empty directory)
///
/// Returns an `MdFile` with path set to an empty `PathBuf` (caller fills it in)
/// and `source` set to a copy of `content`.
pub fn parse_content(content: &str) -> MdFile {
    parse_owned(content.to_owned(), Path::new(""))
}

/// `parse_content` for a caller that already owns the text: it is moved into
/// `MdFile::source` instead of being copied. Link targets are resolved against `file_dir`.
fn parse_owned(source: String, file_dir: &Path) -> MdFile {
    let content = source.as_str();
    // Pre-compute a table mapping line numbers to byte offsets.
    // Index 0 = byte offset of line 1's start (always 0).
    // Index 1 = byte offset of line 2's start (position after first \n).
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    // Total lines = number of actual text lines.
    // A trailing newline does NOT create an additional line.
    // "a\nb\n" = 2 lines; "a\nb" = 2 lines; "" = 0 lines.
    let total_lines = if content.is_empty() {
        0
    } else if content.ends_with('\n') {
        line_starts.len() - 1
    } else {
        line_starts.len()
    };

    // Configure pulldown-cmark with GFM extensions + YAML metadata blocks.
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let parser = Parser::new_ext(content, opts);
    let offset_iter = parser.into_offset_iter();

    // ─── State machine ────────────────────────────────────────────────────────
    let mut entries: Vec<DocEntry> = Vec::new();

    // Current entry being built (None before first MetadataBlock)
    let mut current_entry_sections: Vec<Section> = Vec::new();
    let mut current_entry_fm: Option<Frontmatter> = None;
    let mut current_entry_start_line: usize = 1;
    let mut seen_metadata_block = false;

    // Heading tracking
    let mut in_heading = false;
    let mut heading_level: u8 = 0;
    let mut heading_text = String::new();
    let mut heading_byte_start: usize = 0;

    // MetadataBlock tracking
    let mut in_metadata = false;
    let mut metadata_text = String::new();
    let mut metadata_byte_start: usize = 0;

    // Feature accumulation for the current section
    let mut current_features = FeatureAccumulator::new();

    let mut links: Vec<Link> = Vec::new();

    for (event, range) in offset_iter {
        // Links are recorded before the state machine so that no state (heading,
        // metadata) can hide one. Images and autolinks are other event kinds.
        if let Event::Start(Tag::Link {
            link_type:
                LinkType::Inline | LinkType::Reference | LinkType::Collapsed | LinkType::Shortcut,
            ref dest_url,
            ..
        }) = event
        {
            let line = byte_to_line(&line_starts, range.start);
            links.extend(make_link(dest_url, line, file_dir));
        }

        match event {
            // ─── MetadataBlock (entry boundary) ───────────────────────────
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_metadata = true;
                metadata_text.clear();
                metadata_byte_start = range.start;
            }
            Event::Text(ref text) if in_metadata => {
                metadata_text.push_str(text);
            }
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_metadata = false;
                let fm_line = byte_to_line(&line_starts, metadata_byte_start);

                if seen_metadata_block {
                    // Finalize the previous entry before starting a new one.
                    let prev_end = if fm_line > 1 { fm_line - 1 } else { 1 };
                    finalize_sections(&mut current_entry_sections, prev_end, current_features);
                    entries.push(DocEntry {
                        frontmatter: current_entry_fm.take(),
                        sections: std::mem::take(&mut current_entry_sections),
                        offset: current_entry_start_line,
                        end_offset: prev_end,
                    });
                    current_features = FeatureAccumulator::new();
                }

                seen_metadata_block = true;
                current_entry_start_line = fm_line;
                current_entry_fm = parse_frontmatter(&metadata_text);

                // Any headings collected before the first metadata block
                // precede the first entry and belong to no entry. Discard them.
                if entries.is_empty() && !current_entry_sections.is_empty() {
                    current_entry_sections.clear();
                    current_features = FeatureAccumulator::new();
                }
            }

            // ─── Headings ─────────────────────────────────────────────────
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_level = heading_level_to_u8(level);
                heading_text.clear();
                heading_byte_start = range.start;
            }
            Event::Text(ref text) if in_heading => {
                heading_text.push_str(text);
            }
            Event::Code(ref text) if in_heading => {
                heading_text.push_str(text);
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                let line = byte_to_line(&line_starts, heading_byte_start);

                // Store the accumulated features for the previous section (if any).
                if let Some(prev) = current_entry_sections.last_mut()
                    && prev.features.word_count == 0
                    && !prev.features.has_table
                    && !prev.features.has_code_block
                    && !prev.features.has_list
                {
                    prev.features = std::mem::take(&mut current_features).finish();
                }
                current_features = FeatureAccumulator::new();

                current_entry_sections.push(Section {
                    level: heading_level,
                    title: heading_text.clone(),
                    offset: line,
                    limit: 0, // computed post-hoc by compute_limits()
                    features: ContentFeatures::default(),
                });
            }

            // ─── All other events: feed to feature accumulator ────────────
            ref evt if !in_heading && !in_metadata => {
                current_features.process_event(evt);
            }

            _ => {}
        }
    }

    // ─── Finalize last entry ──────────────────────────────────────────────────
    finalize_sections(&mut current_entry_sections, total_lines, current_features);

    if seen_metadata_block {
        entries.push(DocEntry {
            frontmatter: current_entry_fm,
            sections: current_entry_sections,
            offset: current_entry_start_line,
            end_offset: total_lines,
        });
    } else {
        // No metadata block found -- treat entire file as a single entry with no frontmatter.
        entries.push(DocEntry {
            frontmatter: None,
            sections: current_entry_sections,
            offset: 1,
            end_offset: total_lines,
        });
    }

    MdFile {
        path: std::path::PathBuf::new(),
        entries,
        links,
        total_lines,
        byte_size: content.len() as u64,
        mtime: SystemTime::UNIX_EPOCH,
        source,
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Convert a byte offset in the source string to a 1-based line number.
///
/// Uses binary search (`partition_point`) on the pre-computed `line_starts` table.
/// This is O(log n) where n = number of lines.
///
/// For `line_starts` sorted ascending, `partition_point` gives the count of line
/// starts that are <= our byte offset, which equals the 1-based line number.
fn byte_to_line(line_starts: &[usize], byte_offset: usize) -> usize {
    line_starts.partition_point(|&start| start <= byte_offset)
}

/// Convert pulldown-cmark's `HeadingLevel` enum to a u8 (1-6).
fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Compute section limits post-hoc (matching Python mdd semantics).
///
/// For each section at level L, its limit extends until:
/// - The next heading at level <= L (same or higher importance), minus 1
/// - Or the end of the entry if no such heading follows.
///
/// This means parent sections INCLUDE their children's ranges.
fn compute_limits(sections: &mut [Section], end_of_entry: usize) {
    for i in 0..sections.len() {
        let level = sections[i].level;
        let offset = sections[i].offset;

        // Look forward for the next heading at the same or higher level.
        let end_line = sections[i + 1..]
            .iter()
            .find(|s| s.level <= level)
            .map_or(end_of_entry, |s| s.offset - 1);

        sections[i].limit = end_line - offset + 1;
    }
}

/// Store trailing features on the last section and compute all limits.
fn finalize_sections(
    sections: &mut [Section],
    end_of_entry: usize,
    current_features: FeatureAccumulator,
) {
    // Give the last section any trailing features.
    if let Some(last) = sections.last_mut()
        && last.features.word_count == 0
        && !last.features.has_table
        && !last.features.has_code_block
        && !last.features.has_list
    {
        last.features = current_features.finish();
    }
    compute_limits(sections, end_of_entry);
}

#[cfg(test)]
#[path = "../../tests/common/mod.rs"]
mod strategies;

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::strategies::{arb_level_drop_doc, arb_markdown_doc};
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn source_is_the_raw_text_that_was_parsed() {
        let raw = "\u{feff}---\r\ntype: a\r\n---\r\n\r\n# T\r\n";
        assert_eq!(parse_content(raw).source, raw);
        assert_eq!(parse_content("").source, "");

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("raw.md");
        std::fs::write(&path, raw).unwrap();
        assert_eq!(parse_file(&path).unwrap().source, raw);
    }

    #[test]
    fn byte_to_line_basic() {
        // "line1\nline2\nline3\n"
        // Byte offsets: line1 starts at 0, line2 at 6, line3 at 12
        let content = "line1\nline2\nline3\n";
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(content.match_indices('\n').map(|(i, _)| i + 1))
            .collect();

        assert_eq!(byte_to_line(&line_starts, 0), 1); // start of line 1
        assert_eq!(byte_to_line(&line_starts, 3), 1); // middle of line 1
        assert_eq!(byte_to_line(&line_starts, 6), 2); // start of line 2
        assert_eq!(byte_to_line(&line_starts, 12), 3); // start of line 3
    }

    #[test]
    fn parses_simple_file_with_frontmatter() {
        let content = "---\ntype: reference\ntopic: testing\n---\n\n# Title\n\n## Section One\n\nContent here.\n";
        let md = parse_content(content);

        assert_eq!(md.entries.len(), 1);
        let entry = &md.entries[0];

        // Should have parsed frontmatter
        let fm = entry.frontmatter.as_ref().expect("should have frontmatter");
        assert_eq!(
            fm.fields.get("type").and_then(|v| v.as_str()),
            Some("reference")
        );

        // Should have 2 sections: # Title and ## Section One
        assert_eq!(entry.sections.len(), 2);
        assert_eq!(entry.sections[0].title, "Title");
        assert_eq!(entry.sections[0].level, 1);
        assert_eq!(entry.sections[1].title, "Section One");
        assert_eq!(entry.sections[1].level, 2);
    }

    #[test]
    fn parses_file_without_frontmatter() {
        let content = "# Document Without Frontmatter\n\n## First Heading\n\nSome content.\n\n## Second Heading\n\nMore content.\n";
        let md = parse_content(content);

        assert_eq!(md.entries.len(), 1);
        assert!(md.entries[0].frontmatter.is_none());
        assert_eq!(md.entries[0].sections.len(), 3);
        assert_eq!(
            md.entries[0].sections[0].title,
            "Document Without Frontmatter"
        );
        assert_eq!(md.entries[0].sections[1].title, "First Heading");
        assert_eq!(md.entries[0].sections[2].title, "Second Heading");
    }

    #[test]
    fn section_offsets_and_limits() {
        let content = "---\ntype: test\n---\n\n# Title\n\n## Section One\n\nLine 1.\nLine 2.\n\n## Section Two\n\nLine 3.\n";
        let md = parse_content(content);
        let sections = &md.entries[0].sections;

        // # Title is H1 -- it extends to end of file (no other H1 follows).
        // This matches Python mdd: parent sections encompass their children.
        assert_eq!(sections[0].title, "Title");
        assert_eq!(sections[0].offset, 5);
        assert_eq!(sections[0].limit, 10); // lines 5-14 (entire content after FM)

        // ## Section One: extends until ## Section Two (line 12), so lines 7-11
        assert_eq!(sections[1].title, "Section One");
        assert_eq!(sections[1].offset, 7);
        assert_eq!(sections[1].limit, 5); // lines 7-11

        // ## Section Two: no more H2 follows, so extends to end (line 14)
        assert_eq!(sections[2].title, "Section Two");
        assert_eq!(sections[2].offset, 12);
        assert_eq!(sections[2].limit, 3); // lines 12-14
    }

    #[test]
    fn frontmatter_offset_is_the_one_based_line_of_the_opening_delimiter() {
        let content = include_str!("../../tests/fixtures/multipart-lrn.md");
        let md = parse_content(content);
        let lines: Vec<&str> = content.lines().collect();

        let offsets: Vec<usize> = md
            .entries
            .iter()
            .filter(|e| e.frontmatter.is_some())
            .map(|e| e.offset)
            .collect();
        assert_eq!(offsets, vec![3, 28, 45]);
        for offset in offsets {
            assert_eq!(lines[offset - 1], "---");
        }
    }

    #[test]
    fn section_limits_stay_within_their_entry() {
        let content = include_str!("../../tests/fixtures/multipart-lrn.md");
        let md = parse_content(content);

        assert_eq!(md.entries.len(), 3);
        assert_eq!(md.entries[0].end_offset, 27);
        assert_eq!(md.entries.last().unwrap().end_offset, md.total_lines);

        for entry in &md.entries {
            for s in &entry.sections {
                assert!(
                    s.offset >= entry.offset && s.offset + s.limit - 1 <= entry.end_offset,
                    "section {:?} (offset {}, limit {}) escapes entry {}..={}",
                    s.title,
                    s.offset,
                    s.limit,
                    entry.offset,
                    entry.end_offset
                );
            }
        }

        let thread_pool = &md.entries[0].sections[1];
        assert_eq!(thread_pool.title, "Thread Pool Utilization");
        assert_eq!((thread_pool.offset, thread_pool.limit), (21, 7));
        let last = md.entries[2].sections.last().unwrap();
        assert_eq!(last.offset + last.limit - 1, md.total_lines);
    }

    #[test]
    fn multi_part_document() {
        let content = "\
# Daily 2026-05-10

---
type: reference
topic: testing
weight: 0.8
---

### First Entry

Content of the first entry.

---
type: note
topic: parser
weight: 0.5
---

### Second Entry

Content of the second entry.
";
        let md = parse_content(content);

        // Should have 2 entries (one per metadata block)
        assert_eq!(md.entries.len(), 2);

        // First entry
        let e1 = &md.entries[0];
        let fm1 = e1.frontmatter.as_ref().unwrap();
        assert_eq!(
            fm1.fields.get("type").and_then(|v| v.as_str()),
            Some("reference")
        );
        assert_eq!(e1.sections.len(), 1);
        assert_eq!(e1.sections[0].title, "First Entry");

        // Second entry
        let e2 = &md.entries[1];
        let fm2 = e2.frontmatter.as_ref().unwrap();
        assert_eq!(
            fm2.fields.get("type").and_then(|v| v.as_str()),
            Some("note")
        );
        assert_eq!(e2.sections.len(), 1);
        assert_eq!(e2.sections[0].title, "Second Entry");
    }

    #[test]
    fn detects_content_features() {
        let content = "\
# Title

## Code Section

```rust
fn main() {}
```

## List Section

- item 1
- item 2
- item 3

## Table Section

| A | B |
|---|---|
| 1 | 2 |
";
        let md = parse_content(content);
        let sections = &md.entries[0].sections;

        // Code Section should detect rust code block
        let code_sec = &sections[1];
        assert_eq!(code_sec.title, "Code Section");
        assert!(code_sec.features.has_code_block);
        assert_eq!(code_sec.features.code_languages, vec!["rust"]);

        // List Section should detect list with 3 items
        let list_sec = &sections[2];
        assert_eq!(list_sec.title, "List Section");
        assert!(list_sec.features.has_list);
        assert_eq!(list_sec.features.list_item_count, 3);

        // Table Section should detect table
        let table_sec = &sections[3];
        assert_eq!(table_sec.title, "Table Section");
        assert!(table_sec.features.has_table);
    }

    #[test]
    fn total_lines_counted_correctly() {
        let content = "# One\n\n## Two\n\nThree lines total: five.\n";
        let md = parse_content(content);
        assert_eq!(md.total_lines, 5);
    }

    #[test]
    fn heading_with_inline_code() {
        let content = "# The `main` Function\n\nBody.\n";
        let md = parse_content(content);
        assert_eq!(md.entries[0].sections[0].title, "The main Function");
    }

    #[test]
    fn features_assigned_to_correct_section() {
        let content = "# Title\n\nPlain text.\n\n## With Code\n\n```python\nx = 1\n```\n\n## With List\n\n- a\n- b\n";
        let md = parse_content(content);
        let sections = &md.entries[0].sections;
        assert!(!sections[0].features.has_code_block);
        assert!(!sections[0].features.has_list);
        assert!(sections[1].features.has_code_block);
        assert!(!sections[1].features.has_list);
        assert!(!sections[2].features.has_code_block);
        assert!(sections[2].features.has_list);
        assert_eq!(sections[2].features.list_item_count, 2);
    }

    proptest! {
        #[test]
        fn parsed_sections_are_ordered_and_bounded_by_their_entry(
            doc in prop_oneof![arb_markdown_doc(), arb_level_drop_doc()]
        ) {
            let md = parse_content(&doc);
            prop_assert!(!md.entries.is_empty());
            prop_assert_eq!(md.entries.last().unwrap().end_offset, md.total_lines);

            let mut prev_offset = 0;
            for entry in &md.entries {
                prop_assert!(entry.end_offset <= md.total_lines);
                for s in &entry.sections {
                    prop_assert!(s.offset >= 1, "offset must be >= 1, got {}", s.offset);
                    prop_assert!(s.limit >= 1, "limit must be >= 1, got {}", s.limit);
                    prop_assert!(s.offset > prev_offset, "offsets must ascend: {} after {}", s.offset, prev_offset);
                    prop_assert!(
                        s.offset + s.limit - 1 <= entry.end_offset,
                        "section {:?} (offset {}, limit {}) escapes entry ending at {}",
                        s.title, s.offset, s.limit, entry.end_offset
                    );
                    prev_offset = s.offset;
                }
            }
        }
    }
}
