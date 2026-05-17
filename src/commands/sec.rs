use std::fs;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output::{self, SecMatch, SecOutput};
use crate::parse::markdown::parse_file;
use crate::parse::Section;

pub fn run(path: PathBuf, patterns: Vec<String>, cli: &Cli) -> Result<()> {
    if !path.exists() {
        return Err(MddError::FileNotFound { path });
    }

    let md = parse_file(&path)?;
    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    let content = fs::read_to_string(&path).map_err(|e| MddError::IoError {
        path: path.clone(),
        source: e,
    })?;
    let lines: Vec<&str> = content.lines().collect();

    let mut all_matches: Vec<SecMatch> = Vec::new();
    let mut unmatched: Vec<&str> = Vec::new();

    for pattern in &patterns {
        let pat = clean_pattern(pattern).to_lowercase();
        let matched: Vec<&&Section> = all_sections
            .iter()
            .filter(|s| s.title.to_lowercase().contains(&pat))
            .collect();

        if matched.is_empty() {
            unmatched.push(pattern);
        } else {
            for section in matched {
                let start = section.offset - 1;
                let end = (start + section.limit).min(lines.len());
                let section_content = lines[start..end].join("\n");
                all_matches.push(SecMatch {
                    title: section.title.clone(),
                    offset: section.offset,
                    limit: section.limit,
                    content: section_content,
                });
            }
        }
    }

    if all_matches.is_empty() {
        return Err(MddError::NoMatch {
            pattern: patterns.join(", "),
        });
    }

    if !unmatched.is_empty() {
        eprintln!("warning: no match for: {}", unmatched.join(", "));
    }

    let format = output::resolve_format(&cli.format);
    output::render_sec(
        &format,
        &SecOutput {
            path: &path,
            matches: all_matches,
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
