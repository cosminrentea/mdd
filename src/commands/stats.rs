use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output::fmt_size;
use crate::parse::frontmatter::field_as_string;
use crate::query;

pub fn run(dir: PathBuf, _cli: &Cli) -> Result<()> {
    if !dir.exists() || !dir.is_dir() {
        return Err(MddError::FileNotFound { path: dir });
    }

    let parsed = query::walk_and_parse(&dir);

    if parsed.is_empty() {
        eprintln!("No markdown files found in {}", dir.display());
        return Ok(());
    }

    let mut total_lines = 0usize;
    let mut total_bytes = 0u64;
    let mut with_frontmatter = 0usize;
    let mut with_tables = 0usize;
    let mut with_code = 0usize;
    let mut with_lists = 0usize;
    let mut total_sections = 0usize;
    let mut max_depth: u8 = 0;
    let mut type_counts: BTreeMap<String, usize> = BTreeMap::new();

    for (_path, md) in &parsed {
        total_lines += md.total_lines;
        total_bytes += md.byte_size;

        let has_fm = md.entries.iter().any(|e| e.frontmatter.is_some());
        if has_fm {
            with_frontmatter += 1;
        }

        let mut file_has_table = false;
        let mut file_has_code = false;
        let mut file_has_list = false;

        for entry in &md.entries {
            if let Some(ref fm) = entry.frontmatter {
                if let Some(t) = field_as_string(fm, "type") {
                    *type_counts.entry(t).or_insert(0) += 1;
                }
            }
            for section in &entry.sections {
                total_sections += 1;
                if section.level > max_depth {
                    max_depth = section.level;
                }
                if section.features.has_table {
                    file_has_table = true;
                }
                if section.features.has_code_block {
                    file_has_code = true;
                }
                if section.features.has_list {
                    file_has_list = true;
                }
            }
        }

        if file_has_table {
            with_tables += 1;
        }
        if file_has_code {
            with_code += 1;
        }
        if file_has_list {
            with_lists += 1;
        }
    }

    let file_count = parsed.len();
    let avg_sections = if file_count > 0 {
        total_sections as f64 / file_count as f64
    } else {
        0.0
    };

    let dir_display = dir
        .file_name()
        .unwrap_or(dir.as_os_str())
        .to_string_lossy();

    println!("=== stats for {}/ ===", dir_display);
    println!(
        "Files: {} ({} lines, {})",
        file_count,
        format_number(total_lines),
        fmt_size(total_bytes as usize),
    );

    let coverage_pct = with_frontmatter as f64 / file_count as f64 * 100.0;
    println!(
        "Frontmatter: {}/{} ({:.0}% coverage)",
        with_frontmatter, file_count, coverage_pct,
    );

    if !type_counts.is_empty() {
        let mut sorted: Vec<(&String, &usize)> = type_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        let type_str: Vec<String> = sorted
            .iter()
            .map(|(k, v)| format!("{}({})", k, v))
            .collect();
        println!("  type: {}", type_str.join(", "));
    }

    println!(
        "Content: {} with tables, {} with code, {} with lists",
        with_tables, with_code, with_lists,
    );
    println!(
        "Headings: avg {:.1}/file, max depth {}",
        avg_sections, max_depth,
    );

    Ok(())
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{},{:03}", n / 1000, n % 1000)
    } else {
        n.to_string()
    }
}
