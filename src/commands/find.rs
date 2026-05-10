use std::path::PathBuf;
use std::time::SystemTime;

use chrono::NaiveDate;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output::{self, format_signature, fmt_size};
use crate::parse::frontmatter::field_as_string;
use crate::parse::markdown::parse_file;
use crate::parse::{DocEntry, MdFile, Section};

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
        .map(parse_date_to_system_time)
        .transpose()
        .map_err(|e| MddError::InvalidDate { value: e })?;

    let walker = ignore::WalkBuilder::new(&dir).follow_links(true).build();

    let mut files: Vec<PathBuf> = walker
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "AGENTS.md" | "CLAUDE.local.md" | "CLAUDE.md" | "README.md"
            )
        })
        .map(|e| e.into_path())
        .collect();

    files.sort();

    let format = output::resolve_format(&cli.format);
    let mut match_count = 0;

    for file_path in &files {
        let md = match parse_file(file_path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if let Some(ref cutoff) = newer_cutoff {
            if md.mtime < *cutoff {
                continue;
            }
        }

        let matching_sections = find_matching_sections(
            &md,
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
                    format!("({} lines, {})", md.total_lines, fmt_size(md.byte_size as usize)).dimmed(),
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
    md: &'a MdFile,
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

fn entry_matches_frontmatter(entry: &DocEntry, type_filter: Option<&str>, topic: Option<&str>) -> bool {
    if type_filter.is_none() && topic.is_none() {
        return true;
    }

    let fm = match &entry.frontmatter {
        Some(fm) => fm,
        None => return false,
    };

    if let Some(t) = type_filter {
        match field_as_string(fm, "type") {
            Some(val) => {
                if !val.eq_ignore_ascii_case(t) {
                    return false;
                }
            }
            None => return false,
        }
    }

    if let Some(t) = topic {
        match field_as_string(fm, "topic") {
            Some(val) => {
                if !val.to_lowercase().contains(&t.to_lowercase()) {
                    return false;
                }
            }
            None => return false,
        }
    }

    true
}

fn section_matches(section: &Section, heading: Option<&str>, has_table: bool, has_code: Option<&str>) -> bool {
    if let Some(h) = heading {
        if !section.title.to_lowercase().contains(&h.to_lowercase()) {
            return false;
        }
    }

    if has_table && !section.features.has_table {
        return false;
    }

    if let Some(lang) = has_code {
        if !section.features.has_code_block {
            return false;
        }
        if !lang.is_empty() && !section.features.code_languages.iter().any(|l| l.eq_ignore_ascii_case(lang)) {
            return false;
        }
    }

    true
}

fn parse_date_to_system_time(date_str: &str) -> std::result::Result<SystemTime, String> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("invalid date '{}': {}", date_str, e))?;
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();
    let timestamp = datetime.and_utc().timestamp();
    Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64))
}
