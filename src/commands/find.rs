use chrono::{DateTime, NaiveTime, Utc};

use crate::cli::{FindArgs, OutputFormat};
use crate::error::{MddError, Result};
use crate::output::{fmt_size, format_signature};
use crate::parse::Section;
use crate::query;
use crate::query::filter::Filter;

#[allow(clippy::cast_possible_truncation, reason = "file sizes fit in usize")]
pub fn run(args: &FindArgs, format: OutputFormat) -> Result<()> {
    let dir = &args.dir;
    if !dir.exists() {
        return Err(MddError::FileNotFound { path: dir.clone() });
    }

    let newer_cutoff = args
        .newer_than
        .map(|d| d.and_time(NaiveTime::MIN).and_utc());

    let filter = args.filter();
    let parsed = query::walk_and_parse(dir);
    let mut match_count = 0;

    for (file_path, md) in &parsed {
        if let Some(cutoff) = newer_cutoff
            && DateTime::<Utc>::from(md.mtime) < cutoff
        {
            continue;
        }

        let matching_sections = find_matching_sections(md, &filter);

        if matching_sections.is_empty() {
            continue;
        }

        match_count += 1;
        let rel_path = file_path.strip_prefix(dir).unwrap_or(file_path);

        match format {
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
                        format!(" {sig}")
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
        eprintln!("\n{match_count} files matched");
    }

    Ok(())
}

fn find_matching_sections<'a>(md: &'a crate::parse::MdFile, filter: &Filter) -> Vec<&'a Section> {
    let mut results: Vec<&Section> = Vec::new();

    for entry in &md.entries {
        if !filter.matches_entry(entry) {
            continue;
        }

        for section in &entry.sections {
            if !filter.matches_section(section) {
                continue;
            }
            results.push(section);
        }
    }

    results
}
