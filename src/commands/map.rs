use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::parse::frontmatter::field_as_string;
use crate::parse::markdown::parse_file;
use crate::parse::MdFile;

pub fn run(path: PathBuf, _cli: &Cli) -> Result<()> {
    if !path.exists() || !path.is_dir() {
        return Err(MddError::FileNotFound { path });
    }

    let walker = ignore::WalkBuilder::new(&path).follow_links(true).build();

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

    let now = SystemTime::now();
    let three_days_ago = now - Duration::from_secs(3 * 24 * 3600);

    let mut parsed: Vec<(PathBuf, MdFile)> = Vec::new();
    let mut total_lines: usize = 0;
    let mut hot_files: Vec<(PathBuf, &MdFile)> = Vec::new();

    for file_path in &files {
        match parse_file(file_path) {
            Ok(md) => {
                total_lines += md.total_lines;
                parsed.push((file_path.clone(), md));
            }
            Err(_) => continue,
        }
    }

    // Collect hot files (modified in last 3 days)
    for (fp, md) in &parsed {
        if md.mtime >= three_days_ago {
            hot_files.push((fp.clone(), md));
        }
    }
    hot_files.sort_by_key(|item| Reverse(item.1.mtime));

    // Directory structure stats
    let mut dir_stats: BTreeMap<String, DirInfo> = BTreeMap::new();
    let mut topic_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_with_tables = 0usize;
    let mut total_with_code = 0usize;

    for (fp, md) in &parsed {
        let rel = fp.strip_prefix(&path).unwrap_or(fp);
        let dir_name = rel
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or(".")
            .to_owned();

        let info = dir_stats.entry(dir_name).or_default();
        info.file_count += 1;

        let has_fm = md.entries.iter().any(|e| e.frontmatter.is_some());
        if has_fm {
            info.with_frontmatter += 1;
        }

        for entry in &md.entries {
            if let Some(ref fm) = entry.frontmatter {
                for key in fm.fields.keys() {
                    info.fm_fields.insert(key.clone());
                }
                if let Some(topic) = field_as_string(fm, "topic") {
                    *topic_counts.entry(topic).or_insert(0) += 1;
                }
            }
            for section in &entry.sections {
                if section.features.has_table {
                    total_with_tables += 1;
                }
                if section.features.has_code_block {
                    total_with_code += 1;
                }
            }
        }
    }

    // Print output
    let dir_display = path.file_name().unwrap_or(path.as_os_str()).to_string_lossy();
    println!(
        "=== {}/ ({} files, {}K lines) ===",
        dir_display,
        parsed.len(),
        total_lines / 1000,
    );

    // HOT section
    if !hot_files.is_empty() {
        println!("\nHOT (modified last 3 days):");
        for (fp, md) in hot_files.iter().take(10) {
            let rel = fp.strip_prefix(&path).unwrap_or(fp);
            let headings: Vec<&str> = md
                .entries
                .iter()
                .flat_map(|e| &e.sections)
                .skip(1)
                .take(3)
                .map(|s| s.title.as_str())
                .collect();
            let topic = md
                .entries
                .iter()
                .filter_map(|e| e.frontmatter.as_ref())
                .filter_map(|fm| field_as_string(fm, "topic"))
                .next();

            let mut desc = format!("[{}L]", md.total_lines);
            if let Some(t) = topic {
                desc.push_str(&format!(" topic={}", t));
            }
            if !headings.is_empty() {
                desc.push_str(&format!(
                    " {}",
                    headings.iter().map(|h| format!("## {}", h)).collect::<Vec<_>>().join(", ")
                ));
            }
            println!("  {} {}", rel.display(), desc);
        }
    }

    // STRUCTURE section
    if dir_stats.len() > 1 {
        println!("\nSTRUCTURE:");
        for (dir, info) in &dir_stats {
            if dir == "." {
                continue;
            }
            let coverage = if info.file_count > 0 {
                info.with_frontmatter as f64 / info.file_count as f64 * 100.0
            } else {
                0.0
            };
            let fields_str = if info.fm_fields.is_empty() {
                String::new()
            } else {
                let fields: Vec<&str> = info.fm_fields.iter().map(|s| s.as_str()).collect();
                format!(", fields: {}", fields.join(", "))
            };
            println!(
                "  {}/ ({} files) -- frontmatter: {:.0}%{}",
                dir, info.file_count, coverage, fields_str
            );
        }
    }

    // TOP TOPICS
    if !topic_counts.is_empty() {
        let mut sorted_topics: Vec<(&String, &usize)> = topic_counts.iter().collect();
        sorted_topics.sort_by(|a, b| b.1.cmp(a.1));
        let top: Vec<String> = sorted_topics
            .iter()
            .take(8)
            .map(|(k, v)| format!("{}({})", k, v))
            .collect();
        println!("\nTOP TOPICS: {}", top.join(", "));
    }

    // TOTAL line
    println!(
        "TOTAL: {} files, {} with tables, {} with code blocks",
        parsed.len(),
        total_with_tables,
        total_with_code
    );

    Ok(())
}

#[derive(Default)]
struct DirInfo {
    file_count: usize,
    with_frontmatter: usize,
    fm_fields: BTreeSet<String>,
}
