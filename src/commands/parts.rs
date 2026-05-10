use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output;
use crate::parse::frontmatter::field_as_string;
use crate::parse::markdown::parse_file;

pub fn run(path: PathBuf, filter: Option<String>, cli: &Cli) -> Result<()> {
    if !path.exists() {
        return Err(MddError::FileNotFound { path });
    }

    let md = parse_file(&path)?;
    let format = output::resolve_format(&cli.format);

    if md.entries.is_empty() {
        eprintln!("No entries found in {}", path.display());
        return Ok(());
    }

    // Parse filter: "key=value"
    let filter_kv: Option<(&str, &str)> = filter.as_deref().and_then(|f| f.split_once('='));

    let rel_path = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let mut shown = 0usize;

    for (i, entry) in md.entries.iter().enumerate() {
        // Apply filter if specified
        if let Some((key, value)) = filter_kv {
            match &entry.frontmatter {
                Some(fm) => {
                    let field_val = field_as_string(fm, key).unwrap_or_default();
                    if !field_val.eq_ignore_ascii_case(value) {
                        continue;
                    }
                }
                None => continue,
            }
        }

        shown += 1;
        let entry_limit = entry.end_offset.saturating_sub(entry.offset) + 1;

        // Collect frontmatter summary
        let fm_summary = entry
            .frontmatter
            .as_ref()
            .map(|fm| {
                fm.fields
                    .iter()
                    .filter(|(k, _)| !matches!(k.as_str(), "weight" | "weight-reason" | "agent"))
                    .map(|(k, v)| {
                        let val = match v {
                            serde_yaml::Value::String(s) => s.clone(),
                            other => format!("{:?}", other),
                        };
                        format!("{}={}", k, val)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        // Collect heading names from this entry's sections
        let headings_str = entry
            .sections
            .iter()
            .take(3)
            .map(|s| format!("{} {}", "#".repeat(s.level as usize), s.title))
            .collect::<Vec<_>>()
            .join(", ");

        match &format {
            crate::cli::OutputFormat::Json => {
                let headings: Vec<&str> = entry
                    .sections
                    .iter()
                    .take(3)
                    .map(|s| s.title.as_str())
                    .collect();
                println!(
                    "{}",
                    serde_json::json!({
                        "index": i + 1,
                        "offset": entry.offset,
                        "limit": entry_limit,
                        "frontmatter": fm_summary,
                        "headings": headings,
                    })
                );
            }
            crate::cli::OutputFormat::Agent => {
                let heading_suffix = if headings_str.is_empty() {
                    String::new()
                } else {
                    format!(" {{{}}}", headings_str)
                };
                println!(
                    "[{}:{},{}] Entry {}: {}{}",
                    rel_path,
                    entry.offset,
                    entry_limit,
                    i + 1,
                    fm_summary,
                    heading_suffix,
                );
            }
            crate::cli::OutputFormat::Human => {
                use owo_colors::OwoColorize;
                let heading_suffix = if headings_str.is_empty() {
                    String::new()
                } else {
                    format!(" {{{}}}", headings_str)
                };
                println!(
                    "{} {} {}{}",
                    format!("Entry {}", i + 1).bold(),
                    format!("(offset={}, limit={})", entry.offset, entry_limit).dimmed(),
                    fm_summary,
                    heading_suffix,
                );
            }
        }
    }

    eprintln!("{} entries shown (of {} total)", shown, md.entries.len());
    Ok(())
}
