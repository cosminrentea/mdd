pub mod agent;
pub mod human;
pub mod json;

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

use crate::cli::OutputFormat;
use crate::parse::{ContentFeatures, Section};

pub fn resolve_format(explicit: &Option<OutputFormat>) -> OutputFormat {
    match explicit {
        Some(f) => f.clone(),
        None => {
            if io::stdout().is_terminal() {
                OutputFormat::Human
            } else {
                OutputFormat::Agent
            }
        }
    }
}

// -- TOC output types --

pub struct TocFileOutput<'a> {
    pub path: &'a Path,
    pub sections: Vec<&'a Section>,
    pub min_level: u8,
}

pub struct TocDirEntry {
    pub rel_path: PathBuf,
    pub total_lines: usize,
    pub byte_size: usize,
    pub first_heading: String,
    pub sub_headings: Vec<Section>,
    pub min_level: u8,
}

pub struct TocDirOutput {
    pub entries: Vec<TocDirEntry>,
    pub total_files: usize,
}

// -- SEC output types --

pub struct SecMatch {
    pub title: String,
    pub offset: usize,
    pub limit: usize,
    pub content: String,
}

pub struct SecOutput<'a> {
    pub path: &'a Path,
    pub matches: Vec<SecMatch>,
}

// -- AT output types --

pub struct AtOutput<'a> {
    pub path: &'a Path,
    pub breadcrumb: Vec<String>,
    pub section: &'a Section,
    pub content: String,
}

// -- Rendering dispatch --

pub fn render_toc_file(format: &OutputFormat, output: &TocFileOutput) {
    match format {
        OutputFormat::Human => human::render_toc_file(output),
        OutputFormat::Agent => agent::render_toc_file(output),
        OutputFormat::Json => json::render_toc_file(output),
    }
}

pub fn render_toc_dir(format: &OutputFormat, output: &TocDirOutput) {
    match format {
        OutputFormat::Human => human::render_toc_dir(output),
        OutputFormat::Agent => agent::render_toc_dir(output),
        OutputFormat::Json => json::render_toc_dir(output),
    }
}

pub fn render_sec(format: &OutputFormat, output: &SecOutput<'_>) {
    match format {
        OutputFormat::Human => human::render_sec(output),
        OutputFormat::Agent => agent::render_sec(output),
        OutputFormat::Json => json::render_sec(output),
    }
}

pub fn render_at(format: &OutputFormat, output: &AtOutput<'_>) {
    match format {
        OutputFormat::Human => human::render_at(output),
        OutputFormat::Agent => agent::render_at(output),
        OutputFormat::Json => json::render_at(output),
    }
}

// -- Shared helpers --

pub fn format_signature(features: &ContentFeatures) -> String {
    let mut parts = Vec::new();
    if features.has_table {
        parts.push("tables:1".to_owned());
    }
    if features.has_code_block {
        let langs = if features.code_languages.is_empty() {
            "1".to_owned()
        } else {
            features.code_languages.join(",")
        };
        parts.push(format!("code:{}", langs));
    }
    if features.has_list {
        parts.push(format!("lists:{}", features.list_item_count));
    }
    if features.word_count > 0 {
        parts.push(format!("words:{}", features.word_count));
    }
    if features.link_count > 0 {
        parts.push(format!("links:{}", features.link_count));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("{{{}}}", parts.join(", "))
    }
}

pub fn fmt_size(nbytes: usize) -> String {
    if nbytes >= 1024 * 1024 {
        format!("{:.1}M", nbytes as f64 / (1024.0 * 1024.0))
    } else if nbytes >= 1024 {
        format!("{}K", nbytes / 1024)
    } else {
        format!("{}B", nbytes)
    }
}

