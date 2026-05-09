// "at" command: find the section containing a given line number.
//
// Given a line number:
// 1. Parse the file
// 2. Find the innermost section containing that line
// 3. Climb `level` parents (0 = innermost, 1 = one up, etc.)
// 4. Print the section content
// 5. Print breadcrumb to stderr: [Top > Parent > Section]

use std::fs;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::parse::markdown::parse_file;
use crate::parse::Section;

pub fn run(path: PathBuf, line: usize, level: usize, _cli: &Cli) -> Result<()> {
    if !path.exists() {
        return Err(MddError::FileNotFound { path });
    }

    let md = parse_file(&path)?;

    if line < 1 || line > md.total_lines {
        return Err(MddError::LineOutOfRange {
            line,
            total_lines: md.total_lines,
        });
    }

    // Flatten all sections across entries.
    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    // Find all sections that contain this line.
    // A section contains the line if: section.offset <= line <= section.offset + section.limit - 1
    let containing: Vec<&Section> = all_sections
        .iter()
        .filter(|s| {
            let end = s.offset + s.limit - 1;
            s.offset <= line && line <= end
        })
        .copied()
        .collect();

    if containing.is_empty() {
        // Line is before any heading (in preamble or frontmatter).
        eprintln!("Line {} is before any heading", line);
        return Ok(());
    }

    // The innermost section is the last one in `containing` (deepest nesting).
    let mut target = *containing.last().unwrap();

    // Climb `level` parents.
    for _ in 0..level {
        // Find the nearest containing section with a strictly lower level.
        let parent = containing
            .iter()
            .rev()
            .find(|s| s.level < target.level);
        match parent {
            Some(p) => target = p,
            None => break, // already at the outermost
        }
    }

    // Build breadcrumb: all ancestors of target, plus target itself.
    let ancestry: Vec<&Section> = containing
        .iter()
        .filter(|s| s.level < target.level || std::ptr::eq(**s, target))
        .copied()
        .collect();
    let breadcrumb: String = ancestry.iter().map(|s| s.title.as_str()).collect::<Vec<_>>().join(" > ");
    eprintln!("[{}]", breadcrumb);

    // Read file and print the target section content.
    let content = fs::read_to_string(&path).map_err(|e| MddError::IoError {
        path: path.clone(),
        source: e,
    })?;
    let lines: Vec<&str> = content.lines().collect();

    let start = target.offset - 1;
    let end = (start + target.limit).min(lines.len());

    for l in &lines[start..end] {
        println!("{}", l);
    }

    Ok(())
}
