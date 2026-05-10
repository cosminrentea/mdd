use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::parse::frontmatter::field_as_string;
use crate::parse::markdown::parse_file;

pub fn run(
    path: PathBuf,
    list_values: Option<String>,
    set: Option<String>,
    apply: bool,
    _cli: &Cli,
) -> Result<()> {
    if !path.exists() {
        return Err(MddError::FileNotFound { path });
    }

    if let Some(ref field) = list_values {
        return list_field_values(&path, field);
    }

    if let Some(ref assignment) = set {
        return set_field(&path, assignment, apply);
    }

    show_frontmatter(&path)
}

fn list_field_values(path: &std::path::Path, field: &str) -> Result<()> {
    let mut values = BTreeSet::new();

    if path.is_file() {
        collect_field_values(path, field, &mut values)?;
    } else {
        let walker = ignore::WalkBuilder::new(path).follow_links(true).build();
        let mut files: Vec<PathBuf> = walker
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .map(|e| e.into_path())
            .collect();
        files.sort();

        for file_path in &files {
            let _ = collect_field_values(file_path, field, &mut values);
        }
    }

    if values.is_empty() {
        eprintln!("No values found for field '{}'", field);
    } else {
        for val in &values {
            println!("{}", val);
        }
        eprintln!("\n{} unique values", values.len());
    }

    Ok(())
}

fn collect_field_values(
    path: &std::path::Path,
    field: &str,
    values: &mut BTreeSet<String>,
) -> Result<()> {
    let md = parse_file(path)?;
    for entry in &md.entries {
        if let Some(ref fm) = entry.frontmatter {
            if let Some(val) = field_as_string(fm, field) {
                values.insert(val);
            }
        }
    }
    Ok(())
}

fn show_frontmatter(path: &std::path::Path) -> Result<()> {
    if path.is_file() {
        let md = parse_file(path)?;
        for entry in &md.entries {
            if let Some(ref fm) = entry.frontmatter {
                for (key, val) in &fm.fields {
                    println!("{}: {}", key, format_yaml_value(val));
                }
            }
        }
    } else {
        let walker = ignore::WalkBuilder::new(path).follow_links(true).build();
        let mut files: Vec<PathBuf> = walker
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .map(|e| e.into_path())
            .collect();
        files.sort();

        let mut count = 0;
        let mut with_fm = 0;
        for file_path in &files {
            count += 1;
            let md = match parse_file(file_path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let has_fm = md.entries.iter().any(|e| e.frontmatter.is_some());
            if has_fm {
                with_fm += 1;
            }
        }
        println!(
            "{}/{} files have frontmatter ({:.0}%)",
            with_fm,
            count,
            if count > 0 {
                with_fm as f64 / count as f64 * 100.0
            } else {
                0.0
            }
        );
    }
    Ok(())
}

fn set_field(path: &std::path::Path, assignment: &str, apply: bool) -> Result<()> {
    let (key, value) = assignment
        .split_once('=')
        .ok_or_else(|| MddError::NoMatch {
            pattern: format!("invalid format '{}', expected key=value", assignment),
        })?;

    if !path.is_file() {
        eprintln!("--set only works on single files");
        return Ok(());
    }

    let content = std::fs::read_to_string(path).map_err(|e| MddError::IoError {
        path: path.to_owned(),
        source: e,
    })?;

    let md = parse_file(path)?;
    let fm = md
        .entries
        .first()
        .and_then(|e| e.frontmatter.as_ref())
        .ok_or_else(|| MddError::NoMatch {
            pattern: "file has no frontmatter".to_owned(),
        })?;

    let new_content = apply_frontmatter_set(&content, &fm.raw, fm.offset, key.trim(), value.trim());

    if apply {
        std::fs::write(path, &new_content).map_err(|e| MddError::IoError {
            path: path.to_owned(),
            source: e,
        })?;
        eprintln!("Updated {} in {}", key.trim(), path.display());
    } else {
        eprintln!("Dry run (use --apply to write):");
        println!("{}", new_content);
    }

    Ok(())
}

fn apply_frontmatter_set(
    content: &str,
    _raw_fm: &str,
    _fm_offset: usize,
    key: &str,
    value: &str,
) -> String {
    let mut result: Vec<String> = Vec::new();
    let mut found = false;
    let mut in_fm = false;

    for line in content.lines() {
        if line == "---" {
            if in_fm && !found {
                result.push(format_line(key, value));
                found = true;
            }
            in_fm = !in_fm;
            result.push(line.to_owned());
            continue;
        }

        if in_fm {
            let prefix = format!("{}:", key);
            if line.starts_with(&prefix) {
                result.push(format_line(key, value));
                found = true;
            } else {
                result.push(line.to_owned());
            }
        } else {
            result.push(line.to_owned());
        }
    }

    let mut out = result.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn format_line(key: &str, value: &str) -> String {
    format!("{}: {}", key, value)
}

fn format_yaml_value(val: &serde_yaml::Value) -> String {
    match val {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => "null".to_owned(),
        serde_yaml::Value::Sequence(seq) => {
            let items: Vec<String> = seq.iter().map(format_yaml_value).collect();
            format!("[{}]", items.join(", "))
        }
        serde_yaml::Value::Mapping(_) | serde_yaml::Value::Tagged(_) => serde_yaml::to_string(val)
            .unwrap_or_default()
            .trim()
            .to_owned(),
    }
}
