use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output::{self, fmt_size, format_signature};
use crate::parse::Section;
use crate::query;
use crate::query::filter::entry_matches_frontmatter;
use crate::query::predicates::section_matches;

#[allow(clippy::too_many_arguments)]
pub fn run(
    dir: PathBuf,
    type_filter: Option<String>,
    topic: Option<String>,
    heading: Option<String>,
    has_table: bool,
    has_code: Option<String>,
    newer_than: Option<String>,
    cli: &Cli,
) -> Result<()> {
    if !dir.exists() {
        return Err(MddError::FileNotFound { path: dir });
    }

    let newer_cutoff = newer_than
        .as_deref()
        .map(query::parse_date_to_system_time)
        .transpose()?;

    let parsed = query::walk_and_parse(&dir);
    let format = output::resolve_format(&cli.format);
    let mut match_count = 0;

    for (file_path, md) in &parsed {
        if let Some(ref cutoff) = newer_cutoff {
            if md.mtime < *cutoff {
                continue;
            }
        }

        let matching_sections = find_matching_sections(
            md,
            type_filter.as_deref(),
            topic.as_deref(),
            heading.as_deref(),
            has_table,
            has_code.as_deref(),
        );

        if matching_sections.is_empty() {
            continue;
        }

        match_count += 1;
        let rel_path = file_path.strip_prefix(&dir).unwrap_or(file_path);

        match &format {
            crate::cli::OutputFormat::Json => {
                let sections_json: Vec<serde_json::Value> = matching_sections
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "title": s.title,
                            "level": s.level,
                            "offset": s.offset,
                            "limit": s.limit,
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "path": rel_path.display().to_string(),
                        "total_lines": md.total_lines,
                        "byte_size": md.byte_size,
                        "sections": sections_json,
                    })
                );
            }
            crate::cli::OutputFormat::Agent => {
                for s in &matching_sections {
                    let sig = format_signature(&s.features);
                    let sig_suffix = if sig.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", sig)
                    };
                    println!(
                        "[{}:{},{}] {} {}{}",
                        rel_path.display(),
                        s.offset,
                        s.limit,
                        "#".repeat(s.level as usize),
                        s.title,
                        sig_suffix,
                    );
                }
            }
            crate::cli::OutputFormat::Human => {
                use owo_colors::OwoColorize;
                println!(
                    "{} {}",
                    rel_path.display().to_string().bold(),
                    format!(
                        "({} lines, {})",
                        md.total_lines,
                        fmt_size(md.byte_size as usize)
                    )
                    .dimmed(),
                );
                for s in &matching_sections {
                    let indent = "  ".repeat((s.level.saturating_sub(1)) as usize);
                    println!(
                        "  {}{} {}",
                        indent,
                        s.title,
                        format!("(offset={}, limit={})", s.offset, s.limit).dimmed(),
                    );
                }
            }
        }
    }

    if match_count == 0 {
        eprintln!("No files matched the given filters.");
    } else {
        eprintln!("\n{} files matched", match_count);
    }

    Ok(())
}

fn find_matching_sections<'a>(
    md: &'a crate::parse::MdFile,
    type_filter: Option<&str>,
    topic: Option<&str>,
    heading: Option<&str>,
    has_table: bool,
    has_code: Option<&str>,
) -> Vec<&'a Section> {
    let mut results: Vec<&Section> = Vec::new();

    for entry in &md.entries {
        if !entry_matches_frontmatter(entry, type_filter, topic) {
            continue;
        }

        for section in &entry.sections {
            if !section_matches(section, heading, has_table, has_code) {
                continue;
            }
            results.push(section);
        }
    }

    results
}
