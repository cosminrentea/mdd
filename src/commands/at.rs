use std::fs;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output::{self, AtOutput};
use crate::parse::markdown::parse_file;
use crate::parse::Section;

pub fn run(path: PathBuf, line: usize, level: usize, cli: &Cli) -> Result<()> {
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

    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    let containing: Vec<&Section> = all_sections
        .iter()
        .filter(|s| {
            let end = s.offset + s.limit - 1;
            s.offset <= line && line <= end
        })
        .copied()
        .collect();

    if containing.is_empty() {
        eprintln!("Line {} is before any heading", line);
        return Ok(());
    }

    let mut target = *containing.last().unwrap();

    for _ in 0..level {
        let parent = containing.iter().rev().find(|s| s.level < target.level);
        match parent {
            Some(p) => target = p,
            None => break,
        }
    }

    let ancestry: Vec<&Section> = containing
        .iter()
        .filter(|s| s.level < target.level || std::ptr::eq(**s, target))
        .copied()
        .collect();
    let breadcrumb: Vec<String> = ancestry.iter().map(|s| s.title.clone()).collect();

    let content = fs::read_to_string(&path).map_err(|e| MddError::IoError {
        path: path.clone(),
        source: e,
    })?;
    let lines_vec: Vec<&str> = content.lines().collect();

    let start = target.offset - 1;
    let end = (start + target.limit).min(lines_vec.len());
    let section_content = lines_vec[start..end].join("\n");

    let format = output::resolve_format(&cli.format);
    output::render_at(
        &format,
        &AtOutput {
            path: &path,
            breadcrumb,
            section: target,
            content: section_content,
        },
    );

    Ok(())
}
