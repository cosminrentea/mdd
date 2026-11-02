use std::path::PathBuf;
use std::time::SystemTime;

use super::Outcome;
use crate::cli::{AggArgs, OutputFormat};
use crate::error::{MddError, Result};
use crate::output::format_signature;
use crate::query::{self, Options};

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

pub fn run(args: &AggArgs, format: OutputFormat, opts: Options) -> Result<Outcome> {
    let dir = &args.dir;
    let budget = args.budget;
    if !dir.exists() {
        return Err(MddError::FileNotFound { path: dir.clone() });
    }

    let filter = args.filter();
    let parsed = query::scan(dir, opts)?.parsed;

    // Collect all matching sections with their content
    let mut matches: Vec<AggMatch> = Vec::new();

    for (file_path, md) in &parsed {
        let lines: Vec<&str> = md.source.lines().collect();
        let rel_path = file_path.strip_prefix(dir).unwrap_or(file_path);

        for entry in &md.entries {
            if !filter.matches_entry(entry) {
                continue;
            }

            for section in &entry.sections {
                if !filter.matches_section(section) {
                    continue;
                }

                let content = section.slice(&lines);
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
        return Ok(Outcome::NothingFound(
            "no sections matched the given filters".to_owned(),
        ));
    }

    // Sort: freshest files first, then shorter sections first (more focused)
    matches.sort_by(|a, b| b.mtime.cmp(&a.mtime).then_with(|| a.limit.cmp(&b.limit)));

    // Emit sections until budget exhausted
    let mut lines_used = 0usize;
    let mut emitted = 0usize;

    match format {
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
            println!("{:#}", serde_json::Value::Array(json_sections));
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
                    format!("[{}:{},{}]", m.rel_path.display(), m.offset, m.limit).dimmed(),
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
        eprintln!("--- ({remaining} more sections, {remaining_lines} lines truncated) ---");
    }

    eprintln!("{emitted} sections emitted ({lines_used} lines)");

    Ok(Outcome::Found)
}
