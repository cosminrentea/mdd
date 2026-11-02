// Re-exports and shared data model for the parse module.

pub mod features;
pub mod frontmatter;
pub mod links;
pub mod markdown;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

// ─── Data Model ───────────────────────────────────────────────────────────────
//
// These structs represent the parsed structure of a Markdown file. They are
// produced by `markdown::parse_file` / `markdown::parse_content` and consumed
// by commands (toc, sec, at, find, map, etc.).

/// A fully parsed Markdown file with structural metadata.
#[derive(Debug, Clone)]
pub struct MdFile {
    pub path: PathBuf,
    /// Each logical entry: single-part files have one entry; multi-part (lrn) have many.
    pub entries: Vec<DocEntry>,
    /// Inline, reference, collapsed and shortcut links in the body, in document order.
    pub links: Vec<Link>,
    pub total_lines: usize,
    pub byte_size: u64,
    pub mtime: SystemTime,
    /// The exact text that was parsed (raw: no BOM stripping, no line-ending changes).
    pub source: String,
}

/// One logical entry in a document (a frontmatter block + its subsequent content).
#[derive(Debug, Clone)]
pub struct DocEntry {
    pub frontmatter: Option<Frontmatter>,
    pub sections: Vec<Section>,
    /// Line where this entry starts (1-based).
    pub offset: usize,
    /// Line where this entry ends (1-based, inclusive).
    pub end_offset: usize,
}

/// Parsed YAML frontmatter.
///
/// `BTreeMap` keeps keys sorted, which gives deterministic output
/// when iterating (useful for tests and for predictable `mdd fm` output).
#[derive(Debug, Clone)]
pub struct Frontmatter {
    pub fields: BTreeMap<String, serde_norway::Value>,
}

/// A heading-delimited section of content.
///
/// `offset` and `limit` are designed for direct use with the Read tool:
///   `Read(file_path="...", offset=section.offset, limit=section.limit)`
#[derive(Debug, Clone)]
pub struct Section {
    /// Heading depth (1-6).
    pub level: u8,
    /// The heading text (without the `#` prefix).
    pub title: String,
    /// First line of this section (1-based) -- the heading line itself.
    pub offset: usize,
    /// Number of lines in this section (heading through end of content).
    pub limit: usize,
    /// Summary of what this section contains.
    pub features: ContentFeatures,
}

impl Section {
    /// The section's lines joined with `\n`, taken from `lines` (the file's
    /// `str::lines()`, split once by the caller): `limit` lines starting at the
    /// 1-based `offset`, cut at the end of the file.
    pub fn slice(&self, lines: &[&str]) -> String {
        let start = self.offset - 1;
        let end = (start + self.limit).min(lines.len());
        lines[start..end].join("\n")
    }
}

/// Summary of section content -- lets agents decide whether to drill in.
#[derive(Debug, Clone, Default)]
pub struct ContentFeatures {
    pub has_table: bool,
    pub has_code_block: bool,
    pub code_languages: Vec<String>,
    pub has_list: bool,
    pub list_item_count: u16,
    pub link_count: u16,
    pub word_count: u32,
}

/// A link found in the document.
#[derive(Debug, Clone)]
pub struct Link {
    pub target: String,
    /// Resolved absolute path (if it's a relative file link and resolution succeeded).
    pub resolved_path: Option<PathBuf>,
    /// Line where the link appears (1-based).
    pub line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(offset: usize, limit: usize) -> Section {
        Section {
            level: 1,
            title: String::new(),
            offset,
            limit,
            features: ContentFeatures::default(),
        }
    }

    #[test]
    fn slice_takes_limit_lines_from_the_one_based_offset() {
        let lines = ["# A", "a1", "## B", "b1", "b2"];
        assert_eq!(section(1, 2).slice(&lines), "# A\na1");
        assert_eq!(section(3, 1).slice(&lines), "## B");
    }

    #[test]
    fn slice_of_the_last_section_ends_at_end_of_file() {
        let lines = ["# A", "a1", "## B", "b1", "b2"];
        assert_eq!(section(3, 3).slice(&lines), "## B\nb1\nb2");
    }

    #[test]
    fn slice_with_limit_past_the_end_is_cut_at_end_of_file() {
        let lines = ["# A", "a1"];
        assert_eq!(section(1, 99).slice(&lines), "# A\na1");
        assert_eq!(section(2, 99).slice(&lines), "a1");
    }
}
