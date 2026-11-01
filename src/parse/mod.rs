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
    /// All links found in the document.
    pub links: Vec<Link>,
    pub total_lines: usize,
    pub byte_size: u64,
    pub mtime: SystemTime,
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
    /// Line where the opening `---` appears (1-based).
    pub offset: usize,
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
