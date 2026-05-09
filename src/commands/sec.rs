// Section extraction command.
//
// Given a heading pattern (substring, case-insensitive):
// 1. Parse the file
// 2. Find matching sections
// 3. Print section content using offset/limit
// 4. Multiple matches separated by "---"

use std::fs;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::parse::markdown::parse_file;
use crate::parse::Section;

pub fn run(path: PathBuf, pattern: String, _cli: &Cli) -> Result<()> {
    if !path.exists() {
        return Err(MddError::FileNotFound { path });
    }

    let md = parse_file(&path)?;

    // Flatten all sections across entries.
    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    // Clean pattern: strip toc annotations and leading "- "
    let pat = clean_pattern(&pattern).to_lowercase();

    // Find matching sections
    let matches: Vec<&Section> = all_sections
        .into_iter()
        .filter(|s| s.title.to_lowercase().contains(&pat))
        .collect();

    if matches.is_empty() {
        return Err(MddError::NoMatch { pattern });
    }

    // Read the file lines for content extraction.
    let content = fs::read_to_string(&path).map_err(|e| MddError::IoError {
        path: path.clone(),
        source: e,
    })?;
    let lines: Vec<&str> = content.lines().collect();

    let mut first = true;
    for section in &matches {
        if !first {
            println!("\n---\n");
        }
        first = false;

        // offset is 1-based, limit is the line count
        let start = section.offset - 1; // convert to 0-based index
        let end = (start + section.limit).min(lines.len());

        for line in &lines[start..end] {
            println!("{}", line);
        }
    }

    Ok(())
}

/// Strip leading "- " and trailing "(offset=N, limit=M)" from a pattern.
fn clean_pattern(pat: &str) -> String {
    let mut s = pat.trim().to_owned();
    if let Some(stripped) = s.strip_prefix("- ") {
        s = stripped.to_owned();
    }
    if let Some(idx) = s.rfind(" (offset=") {
        s.truncate(idx);
    }
    s
}
