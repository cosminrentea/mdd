use std::path::PathBuf;
use std::time::SystemTime;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output::{self, format_signature};
use crate::query;
use crate::query::filter::entry_matches_frontmatter;
use crate::query::predicates::section_matches;

/// A matched section ready for output, carrying its content and provenance.
struct AggMatch {
    rel_path: PathBuf,
    title: String,
    level: u8,
    offset: usize,
    limit: usize,
    content: String,
    signature: String,
    mtime: SystemTime,
}

pub fn run(
    dir: PathBuf,
    heading: Option<String>,
    type_filter: Option<String>,
    topic: Option<String>,
    budget: usize,
    cli: &Cli,
) -> Result<()> {
    if !dir.exists() {
        return Err(MddError::FileNotFound { path: dir.clone() });
    }

    let parsed = query::walk_and_parse(&dir);
    let format = output::resolve_format(&cli.format);

    // Collect all matching sections with their content
    let mut matches: Vec<AggMatch> = Vec::new();

    for (file_path, md) in &parsed {
        let file_content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = file_content.lines().collect();
        let rel_path = file_path.strip_prefix(&dir).unwrap_or(file_path);

        for entry in &md.entries {
            if !entry_matches_frontmatter(entry, type_filter.as_deref(), topic.as_deref()) {
                continue;
            }

            for section in &entry.sections {
                if !section_matches(section, heading.as_deref(), false, None) {
                    continue;
                }

                let start = section.offset.saturating_sub(1);
                let end = (start + section.limit).min(lines.len());
                let content = lines[start..end].join("\n");
                let signature = format_signature(&section.features);

                matches.push(AggMatch {
                    rel_path: rel_path.to_path_buf(),
                    title: section.title.clone(),
                    level: section.level,
                    offset: section.offset,
                    limit: section.limit,
                    content,
                    signature,
                    mtime: md.mtime,
                });
            }
        }
    }

    if matches.is_empty() {
        eprintln!("No sections matched the given filters.");
        return Ok(());
    }

    // Sort: freshest files first, then shorter sections first (more focused)
    matches.sort_by(|a, b| {
        b.mtime
            .cmp(&a.mtime)
            .then_with(|| a.limit.cmp(&b.limit))
    });

    // Emit sections until budget exhausted
    let mut lines_used = 0usize;
    let mut emitted = 0usize;

    match &format {
        crate::cli::OutputFormat::Json => {
            let mut json_sections: Vec<serde_json::Value> = Vec::new();
            for m in &matches {
                if lines_used + m.limit > budget && emitted > 0 {
                    break;
                }
                json_sections.push(serde_json::json!({
                    "path": m.rel_path.display().to_string(),
                    "title": m.title,
                    "offset": m.offset,
                    "limit": m.limit,
                    "content": m.content,
                }));
                lines_used += m.limit;
                emitted += 1;
            }
            println!("{}", serde_json::to_string_pretty(&json_sections).unwrap());
        }
        crate::cli::OutputFormat::Agent => {
            for m in &matches {
                if lines_used + m.limit > budget && emitted > 0 {
                    break;
                }
                let sig_suffix = if m.signature.is_empty() {
                    String::new()
                } else {
                    format!(" {}", m.signature)
                };
                println!(
                    "[{}:{},{}] {} {}{}",
                    m.rel_path.display(),
                    m.offset,
                    m.limit,
                    "#".repeat(m.level as usize),
                    m.title,
                    sig_suffix,
                );
                println!("{}\n", m.content);
                lines_used += m.limit;
                emitted += 1;
            }
        }
        crate::cli::OutputFormat::Human => {
            use owo_colors::OwoColorize;
            for m in &matches {
                if lines_used + m.limit > budget && emitted > 0 {
                    break;
                }
                println!(
                    "{} {}",
                    format!(
                        "[{}:{},{}]",
                        m.rel_path.display(),
                        m.offset,
                        m.limit
                    )
                    .dimmed(),
                    m.title.bold(),
                );
                println!("{}\n", m.content);
                lines_used += m.limit;
                emitted += 1;
            }
        }
    }

    // Truncation notice
    let remaining = matches.len() - emitted;
    if remaining > 0 {
        let remaining_lines: usize = matches[emitted..].iter().map(|m| m.limit).sum();
        eprintln!(
            "--- ({} more sections, {} lines truncated) ---",
            remaining, remaining_lines
        );
    }

    eprintln!("{} sections emitted ({} lines)", emitted, lines_used);

    Ok(())
}
