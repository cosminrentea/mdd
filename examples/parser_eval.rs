//! Parser Evaluation -- Task 0
//!
//! Compares pulldown-cmark, comrak, and markdown-rs (wooorm) for our use case:
//! 1. Heading extraction with line/byte offsets
//! 2. Frontmatter detection and extraction
//! 3. Multi-part document handling (multiple --- blocks)
//! 4. GFM features (tables, code blocks with language)
//! 5. Performance on 320 files
//!
//! Run with: cargo run --example parser_eval
//!
//! # Rust concepts demonstrated:
//! - `use` imports and crate namespacing
//! - String slices (&str) vs owned Strings
//! - Iterators and closures
//! - Pattern matching on enums
//! - std::time for benchmarking

use std::fs;
use std::path::Path;
use std::time::Instant;

// ============================================================
// PARSER 1: pulldown-cmark (streaming/pull, byte offsets)
// ============================================================

/// Extract headings using pulldown-cmark's OffsetIter.
///
/// pulldown-cmark is a "pull parser": you iterate over events one by one.
/// Each event has a byte range (Range<usize>) indicating where in the source
/// string that event came from. We use this to track heading positions.
fn pulldown_extract_headings(content: &str) -> Vec<(u8, String, usize, usize)> {
    use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

    // Enable GFM extensions + YAML metadata blocks
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    // OffsetIter wraps Parser and yields (Event, Range<usize>)
    // The Range is the byte offset span in the source string.
    let parser = Parser::new_ext(content, opts);
    let mut offset_iter = parser.into_offset_iter();

    // Pre-compute line starts for byte-offset -> line-number conversion.
    // This is a Vec where index = line number (0-based), value = byte offset of line start.
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(content.match_indices('\n').map(|(i, _)| i + 1))
        .collect();

    let mut headings: Vec<(u8, String, usize, usize)> = Vec::new();
    let mut in_heading = false;
    let mut current_level: u8 = 0;
    let mut current_text = String::new();
    let mut heading_byte_start: usize = 0;

    while let Some((event, range)) = offset_iter.next() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                current_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                current_text.clear();
                heading_byte_start = range.start;
            }
            Event::Text(text) if in_heading => {
                current_text.push_str(&text);
            }
            Event::End(TagEnd::Heading(_)) => {
                // Convert byte offset to 1-based line number
                let line = line_starts.partition_point(|&start| start <= heading_byte_start);
                headings.push((current_level, current_text.clone(), line, heading_byte_start));
                in_heading = false;
            }
            _ => {}
        }
    }

    headings
}

/// Check if pulldown-cmark detects frontmatter (YAML metadata block).
fn pulldown_detect_frontmatter(content: &str) -> Option<String> {
    use pulldown_cmark::{Event, MetadataBlockKind, Options, Parser, Tag, TagEnd};

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);

    let parser = Parser::new_ext(content, opts);
    let mut in_metadata = false;
    let mut metadata_content = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                in_metadata = true;
                metadata_content.clear();
            }
            Event::Text(text) if in_metadata => {
                metadata_content.push_str(&text);
            }
            Event::End(TagEnd::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                return Some(metadata_content);
            }
            _ => {}
        }
    }

    None
}

// ============================================================
// PARSER 2: comrak (DOM tree, line/column positions)
// ============================================================

/// Extract headings using comrak's arena-allocated AST.
///
/// comrak builds a full document tree (like a DOM). You traverse nodes
/// to find headings. Each node has a `sourcepos` with start/end line+column.
fn comrak_extract_headings(content: &str) -> Vec<(u8, String, usize, usize)> {
    use comrak::nodes::NodeValue;
    use comrak::{parse_document, Arena, Options};

    let arena = Arena::new();
    let mut opts = Options::default();
    opts.extension.table = true;
    opts.extension.strikethrough = true;
    opts.extension.tasklist = true;
    opts.extension.front_matter_delimiter = Some("---".to_string());

    let root = parse_document(&arena, content, &opts);

    let mut headings: Vec<(u8, String, usize, usize)> = Vec::new();

    // Iterate all descendants of the root node
    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::Heading(ref heading) = data.value {
            // Collect text content from child nodes
            let mut text = String::new();
            for child in node.children() {
                let child_data = child.data.borrow();
                if let NodeValue::Text(ref t) = child_data.value {
                    text.push_str(t);
                }
            }
            let line = data.sourcepos.start.line;
            let col = data.sourcepos.start.column;
            headings.push((heading.level, text, line, col));
        }
    }

    headings
}

/// Check if comrak detects frontmatter.
fn comrak_detect_frontmatter(content: &str) -> Option<String> {
    use comrak::nodes::NodeValue;
    use comrak::{parse_document, Arena, Options};

    let arena = Arena::new();
    let mut opts = Options::default();
    opts.extension.front_matter_delimiter = Some("---".to_string());

    let root = parse_document(&arena, content, &opts);

    for node in root.descendants() {
        let data = node.data.borrow();
        if let NodeValue::FrontMatter(ref fm) = data.value {
            return Some(fm.trim().to_string());
        }
    }

    None
}

// ============================================================
// PARSER 3: markdown-rs (mdast tree, character offsets)
// ============================================================

/// Extract headings using the `markdown` crate's mdast output.
///
/// markdown-rs (wooorm) produces an mdast-compliant AST. Each node has a
/// Position with start/end Points containing line, column, AND character offset.
fn markdown_rs_extract_headings(content: &str) -> Vec<(u8, String, usize, usize)> {
    use markdown::mdast::Node;
    use markdown::{to_mdast, ParseOptions};

    let mut parse_opts = ParseOptions::gfm();
    parse_opts.constructs.frontmatter = true;

    let tree = match to_mdast(content, &parse_opts) {
        Ok(tree) => tree,
        Err(_) => return Vec::new(),
    };

    let mut headings: Vec<(u8, String, usize, usize)> = Vec::new();

    fn collect_text(node: &Node) -> String {
        match node {
            Node::Text(t) => t.value.clone(),
            Node::InlineCode(c) => c.value.clone(),
            _ => {
                if let Some(children) = node.children() {
                    children.iter().map(collect_text).collect::<Vec<_>>().join("")
                } else {
                    String::new()
                }
            }
        }
    }

    fn walk(node: &Node, headings: &mut Vec<(u8, String, usize, usize)>) {
        if let Node::Heading(h) = node {
            let text = h
                .children
                .iter()
                .map(|c| collect_text(c))
                .collect::<Vec<_>>()
                .join("");
            let (line, offset) = h
                .position
                .as_ref()
                .map(|p| (p.start.line, p.start.offset))
                .unwrap_or((0, 0));
            headings.push((h.depth, text, line, offset));
        }
        if let Some(children) = node.children() {
            for child in children {
                walk(child, headings);
            }
        }
    }

    walk(&tree, &mut headings);
    headings
}

/// Check if markdown-rs detects frontmatter.
fn markdown_rs_detect_frontmatter(content: &str) -> Option<String> {
    use markdown::mdast::Node;
    use markdown::{to_mdast, ParseOptions};

    let mut parse_opts = ParseOptions::gfm();
    parse_opts.constructs.frontmatter = true;

    let tree = match to_mdast(content, &parse_opts) {
        Ok(tree) => tree,
        Err(_) => return None,
    };

    fn find_yaml(node: &Node) -> Option<String> {
        if let Node::Yaml(y) = node {
            return Some(y.value.clone());
        }
        if let Some(children) = node.children() {
            for child in children {
                if let Some(found) = find_yaml(child) {
                    return Some(found);
                }
            }
        }
        None
    }

    find_yaml(&tree)
}

// ============================================================
// MULTI-PART DOCUMENT TEST
// ============================================================

/// Test how each parser handles a document with MULTIPLE --- blocks.
/// Our lrn files have: # Title \n\n---\nyaml\n---\n\n### Content \n\n---\nyaml\n---\n\n### More
fn test_multipart(content: &str) {
    println!("\n--- MULTI-PART DOCUMENT TEST ---");

    // Count --- lines in the content
    let separator_count = content.lines().filter(|l| l.trim() == "---").count();
    println!("  Separator lines (---) in file: {}", separator_count);

    // pulldown-cmark: how many MetadataBlock events does it emit?
    {
        use pulldown_cmark::{Event, MetadataBlockKind, Options, Parser, Tag, TagEnd};
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        let parser = Parser::new_ext(content, opts);

        let mut metadata_count = 0;
        let mut thematic_break_count = 0;
        for event in parser {
            match event {
                Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)) => {
                    metadata_count += 1;
                }
                Event::Rule => {
                    thematic_break_count += 1;
                }
                _ => {}
            }
        }
        println!(
            "  pulldown-cmark: {} metadata blocks, {} thematic breaks",
            metadata_count, thematic_break_count
        );
    }

    // comrak: how many FrontMatter nodes?
    {
        use comrak::nodes::NodeValue;
        use comrak::{parse_document, Arena, Options};
        let arena = Arena::new();
        let mut opts = Options::default();
        opts.extension.front_matter_delimiter = Some("---".to_string());
        let root = parse_document(&arena, content, &opts);

        let mut fm_count = 0;
        let mut tb_count = 0;
        for node in root.descendants() {
            let data = node.data.borrow();
            match data.value {
                NodeValue::FrontMatter(_) => fm_count += 1,
                NodeValue::ThematicBreak => tb_count += 1,
                _ => {}
            }
        }
        println!(
            "  comrak:         {} front matter nodes, {} thematic breaks",
            fm_count, tb_count
        );
    }

    // markdown-rs: how many Yaml nodes?
    {
        use markdown::mdast::Node;
        use markdown::{to_mdast, ParseOptions};
        let mut parse_opts = ParseOptions::gfm();
        parse_opts.constructs.frontmatter = true;

        if let Ok(tree) = to_mdast(content, &parse_opts) {
            let mut yaml_count = 0;
            let mut tb_count = 0;
            fn count_nodes(node: &Node, yaml: &mut usize, tb: &mut usize) {
                match node {
                    Node::Yaml(_) => *yaml += 1,
                    Node::ThematicBreak(_) => *tb += 1,
                    _ => {}
                }
                if let Some(children) = node.children() {
                    for child in children {
                        count_nodes(child, yaml, tb);
                    }
                }
            }
            count_nodes(&tree, &mut yaml_count, &mut tb_count);
            println!(
                "  markdown-rs:    {} yaml nodes, {} thematic breaks",
                yaml_count, tb_count
            );
        } else {
            println!("  markdown-rs:    PARSE ERROR");
        }
    }
}

// ============================================================
// BENCHMARK
// ============================================================

fn benchmark_directory(dir: &Path) {
    println!("\n--- PERFORMANCE BENCHMARK ---");
    println!("  Directory: {}", dir.display());

    let files: Vec<_> = ignore::WalkBuilder::new(dir)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "md")
        })
        .map(|e| e.path().to_owned())
        .collect();

    println!("  Files found: {}", files.len());

    let contents: Vec<String> = files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .collect();

    let total_bytes: usize = contents.iter().map(|c| c.len()).sum();
    println!("  Total bytes: {} ({:.1} KB)", total_bytes, total_bytes as f64 / 1024.0);

    // Benchmark pulldown-cmark
    let start = Instant::now();
    let mut heading_count = 0;
    for content in &contents {
        heading_count += pulldown_extract_headings(content).len();
    }
    let pulldown_time = start.elapsed();
    println!(
        "  pulldown-cmark: {:?} ({} headings)",
        pulldown_time, heading_count
    );

    // Benchmark comrak
    let start = Instant::now();
    let mut heading_count = 0;
    for content in &contents {
        heading_count += comrak_extract_headings(content).len();
    }
    let comrak_time = start.elapsed();
    println!(
        "  comrak:         {:?} ({} headings)",
        comrak_time, heading_count
    );

    // Benchmark markdown-rs
    let start = Instant::now();
    let mut heading_count = 0;
    for content in &contents {
        heading_count += markdown_rs_extract_headings(content).len();
    }
    let markdown_rs_time = start.elapsed();
    println!(
        "  markdown-rs:    {:?} ({} headings)",
        markdown_rs_time, heading_count
    );

    // Ratios
    println!("\n  Speed ratios (lower is faster):");
    let base = pulldown_time.as_nanos() as f64;
    println!("    pulldown-cmark: 1.0x (baseline)");
    println!(
        "    comrak:         {:.1}x",
        comrak_time.as_nanos() as f64 / base
    );
    println!(
        "    markdown-rs:    {:.1}x",
        markdown_rs_time.as_nanos() as f64 / base
    );
}

// ============================================================
// MAIN
// ============================================================

fn main() {
    println!("=== mdd Parser Evaluation (Task 0) ===\n");

    // Test files
    let test_files = [
        ("simple (with frontmatter)", "tests/fixtures/simple.md"),
        ("no frontmatter", "tests/fixtures/no-frontmatter.md"),
        ("lrn multi-part", "tests/fixtures/lrn-multipart.md"),
        ("memory index", "tests/fixtures/memory-index.md"),
    ];

    for (label, path) in &test_files {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                println!("  SKIP {}: {}", path, e);
                continue;
            }
        };

        println!("=== File: {} ({}) ===", label, path);
        println!("  Lines: {}, Bytes: {}", content.lines().count(), content.len());

        // Heading extraction comparison
        println!("\n  Headings (level, text, line, offset):");
        let ph = pulldown_extract_headings(&content);
        let ch = comrak_extract_headings(&content);
        let mh = markdown_rs_extract_headings(&content);

        println!("    pulldown-cmark ({} found):", ph.len());
        for (level, text, line, byte_off) in &ph {
            println!("      H{} \"{}\" line={} byte={}", level, text, line, byte_off);
        }
        println!("    comrak ({} found):", ch.len());
        for (level, text, line, col) in &ch {
            println!("      H{} \"{}\" line={} col={}", level, text, line, col);
        }
        println!("    markdown-rs ({} found):", mh.len());
        for (level, text, line, char_off) in &mh {
            println!("      H{} \"{}\" line={} charoff={}", level, text, line, char_off);
        }

        // Frontmatter detection
        println!("\n  Frontmatter:");
        let pf = pulldown_detect_frontmatter(&content);
        let cf = comrak_detect_frontmatter(&content);
        let mf = markdown_rs_detect_frontmatter(&content);
        println!(
            "    pulldown-cmark: {}",
            pf.as_ref().map(|s| format!("YES ({} bytes)", s.len())).unwrap_or("NONE".into())
        );
        println!(
            "    comrak:         {}",
            cf.as_ref().map(|s| format!("YES ({} bytes)", s.len())).unwrap_or("NONE".into())
        );
        println!(
            "    markdown-rs:    {}",
            mf.as_ref().map(|s| format!("YES ({} bytes)", s.len())).unwrap_or("NONE".into())
        );

        println!();
    }

    // Multi-part test (the critical test for lrn format)
    println!("--- Multi-part: single entry (lrn-multipart.md) ---");
    let lrn_content = fs::read_to_string("tests/fixtures/lrn-multipart.md")
        .expect("lrn fixture exists");
    test_multipart(&lrn_content);

    println!("\n--- Multi-part: TWO entries (lrn-two-entries.md) ---");
    let two_content = fs::read_to_string("tests/fixtures/lrn-two-entries.md")
        .expect("two-entry fixture exists");
    test_multipart(&two_content);

    // Performance benchmark on the real obs/ vault
    let obs_path = Path::new("/Users/rentea/commerce/obs");
    if obs_path.exists() {
        benchmark_directory(obs_path);
    } else {
        println!("\n  SKIP benchmark: obs/ directory not found");
    }

    println!("\n=== Evaluation complete ===");
}
