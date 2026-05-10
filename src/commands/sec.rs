use std::fs;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output::{self, SecMatch, SecOutput};
use crate::parse::markdown::parse_file;
use crate::parse::Section;

pub fn run(path: PathBuf, pattern: String, cli: &Cli) -> Result<()> {
    if !path.exists() {
        return Err(MddError::FileNotFound { path });
    }

    let md = parse_file(&path)?;
    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    let pat = clean_pattern(&pattern).to_lowercase();
    let matches: Vec<&Section> = all_sections
        .into_iter()
        .filter(|s| s.title.to_lowercase().contains(&pat))
        .collect();

    if matches.is_empty() {
        return Err(MddError::NoMatch { pattern });
    }

    let content = fs::read_to_string(&path).map_err(|e| MddError::IoError {
        path: path.clone(),
        source: e,
    })?;
    let lines: Vec<&str> = content.lines().collect();

    let sec_matches: Vec<SecMatch> = matches
        .iter()
        .map(|section| {
            let start = section.offset - 1;
            let end = (start + section.limit).min(lines.len());
            let section_content = lines[start..end].join("\n");
            SecMatch {
                title: section.title.clone(),
                offset: section.offset,
                limit: section.limit,
                content: section_content,
            }
        })
        .collect();

    let format = output::resolve_format(&cli.format);
    output::render_sec(
        &format,
        &SecOutput {
            path: &path,
            matches: sec_matches,
        },
    );

    Ok(())
}

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
