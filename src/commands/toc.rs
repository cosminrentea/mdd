// Table of contents command.
//
// Produces output matching the Python mdd format:
//   - Section One (offset=8, limit=7)
//     - Subsection (offset=10, limit=3)
//
// Works on both single files and directories.
// For directories: walks .md files and shows per-file headings.

use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::parse::markdown::{parse_content, parse_file};
use crate::parse::Section;

/// Run the toc command.
///
/// - If `path` is a file: show its heading outline with offset/limit.
/// - If `path` is a directory: show per-file outlines (like Python `mdd DIR dir`).
/// - If `pattern` is provided: filter to matching headings only.
pub fn run(path: PathBuf, pattern: Option<String>, _cli: &Cli) -> Result<()> {
    if path.is_dir() {
        run_dir(&path, pattern.as_deref())
    } else {
        run_file(&path, pattern.as_deref()) // PathBuf auto-derefs to &Path
    }
}

/// TOC for a single file.
fn run_file(path: &std::path::Path, pattern: Option<&str>) -> Result<()> {
    let md = parse_file(path)?;

    // Collect all sections across all entries into a flat list.
    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    if all_sections.is_empty() {
        return Err(MddError::NoHeadings {
            path: path.to_owned(),
        });
    }

    // Apply pattern filter (substring, case-insensitive).
    let filtered: Vec<&Section> = if let Some(pat) = pattern {
        let pat_lower = clean_toc_pattern(pat).to_lowercase();
        let matches: Vec<&Section> = all_sections
            .into_iter()
            .filter(|s| s.title.to_lowercase().contains(&pat_lower))
            .collect();
        if matches.is_empty() {
            return Err(MddError::NoMatch {
                pattern: pat.to_owned(),
            });
        }
        matches
    } else {
        all_sections
    };

    // Determine minimum level for indentation (just like Python tool).
    let min_level = filtered.iter().map(|s| s.level).min().unwrap_or(1);

    for section in &filtered {
        let indent = "  ".repeat((section.level - min_level) as usize);
        println!(
            "{}- {} (offset={}, limit={})",
            indent, section.title, section.offset, section.limit
        );
    }

    Ok(())
}

/// TOC for a directory: walk .md files, show per-file headings.
fn run_dir(dir: &std::path::Path, pattern: Option<&str>) -> Result<()> {
    if !dir.exists() {
        return Err(MddError::FileNotFound { path: dir.to_owned() });
    }

    // Use the `ignore` crate for .gitignore-aware walking.
    let walker = ignore::WalkBuilder::new(dir)
        .follow_links(true)
        .build();

    let mut files: Vec<PathBuf> = walker
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "md")
        })
        .filter(|e| {
            // Skip common agent/config files (matching Python behavior)
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "AGENTS.md" | "CLAUDE.local.md" | "CLAUDE.md" | "README.md"
            )
        })
        .map(|e| e.into_path())
        .collect();

    files.sort();

    let pat_lower = pattern.map(|p| clean_toc_pattern(p).to_lowercase());
    let mut total_files = 0;

    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let md = parse_content(&content);
        let all_sections: Vec<&Section> =
            md.entries.iter().flat_map(|e| &e.sections).collect();

        let first_heading = all_sections
            .first()
            .map(|s| s.title.as_str())
            .unwrap_or("(no headings)");

        // For directory mode, show sub-headings (depth 1 below the title).
        let min_level = all_sections.first().map(|s| s.level).unwrap_or(1);
        let sub_headings: Vec<&Section> = all_sections
            .iter()
            .skip(1)
            .filter(|s| s.level - min_level <= 1)
            .copied()
            .collect();

        // Pattern filtering
        if let Some(ref pat) = pat_lower {
            let rel = file_path
                .strip_prefix(dir)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_lowercase();
            let heading_match = first_heading.to_lowercase().contains(pat.as_str());
            let sub_match = sub_headings
                .iter()
                .any(|s| s.title.to_lowercase().contains(pat.as_str()));
            if !rel.contains(pat.as_str()) && !heading_match && !sub_match {
                continue;
            }
        }

        total_files += 1;
        let rel_path = file_path.strip_prefix(dir).unwrap_or(file_path);
        let byte_size = content.len();
        println!(
            "{} ({} lines, {}) -- {}",
            rel_path.display(),
            md.total_lines,
            fmt_size(byte_size),
            first_heading
        );

        for s in &sub_headings {
            let indent = "  ".repeat((s.level - min_level) as usize);
            println!("  {}{}", indent, s.title);
        }
    }

    if total_files == 0 {
        if let Some(pat) = pattern {
            return Err(MddError::NoMatch {
                pattern: pat.to_owned(),
            });
        }
    }

    eprintln!("\n{} files", total_files);
    Ok(())
}

/// Strip leading "- " and trailing "(offset=N, limit=M)" from a pattern.
/// This allows pasting toc output directly as a pattern for sec/toc filtering.
fn clean_toc_pattern(pat: &str) -> String {
    let mut s = pat.trim().to_owned();
    if let Some(stripped) = s.strip_prefix("- ") {
        s = stripped.to_owned();
    }
    // Remove trailing (offset=N, limit=M)
    if let Some(idx) = s.rfind(" (offset=") {
        s.truncate(idx);
    }
    s
}

fn fmt_size(nbytes: usize) -> String {
    if nbytes >= 1024 * 1024 {
        format!("{:.1}M", nbytes as f64 / (1024.0 * 1024.0))
    } else if nbytes >= 1024 {
        format!("{}K", nbytes / 1024)
    } else {
        format!("{}B", nbytes)
    }
}
