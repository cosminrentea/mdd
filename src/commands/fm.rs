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
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .map(ignore::DirEntry::into_path)
            .collect();
        files.sort();

        for file_path in &files {
            let _ = collect_field_values(file_path, field, &mut values);
        }
    }

    if values.is_empty() {
        eprintln!("No values found for field '{field}'");
    } else {
        for val in &values {
            println!("{val}");
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
        if let Some(ref fm) = entry.frontmatter
            && let Some(val) = field_as_string(fm, field)
        {
            values.insert(val);
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
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .map(ignore::DirEntry::into_path)
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
                f64::from(with_fm) / f64::from(count) * 100.0
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
            pattern: format!("invalid format '{assignment}', expected key=value"),
        })?;

    if !path.is_file() {
        return Err(MddError::NoMatch {
            pattern: "--set only works on single files".to_owned(),
        });
    }

    let content = std::fs::read_to_string(path).map_err(|e| MddError::IoError {
        path: path.to_owned(),
        source: e,
    })?;

    let md = parse_file(path)?;
    md.entries
        .first()
        .and_then(|e| e.frontmatter.as_ref())
        .ok_or_else(|| MddError::NoMatch {
            pattern: "file has no frontmatter".to_owned(),
        })?;
    let block_starts: Vec<usize> = md
        .entries
        .iter()
        .filter_map(|e| e.frontmatter.as_ref().map(|fm| fm.offset))
        .collect();

    let new_content = apply_frontmatter_set(&content, &block_starts, key.trim(), value.trim())?;

    if apply {
        if new_content == content {
            eprintln!("No change to {} in {}", key.trim(), path.display());
            return Ok(());
        }
        std::fs::write(path, &new_content).map_err(|e| MddError::IoError {
            path: path.to_owned(),
            source: e,
        })?;
        eprintln!("Updated {} in {}", key.trim(), path.display());
    } else {
        eprintln!("Dry run (use --apply to write):");
        println!("{new_content}");
    }

    Ok(())
}

/// Rewrites `key` only in the parser-located blocks (1-based opening `---` lines).
/// An absent key is appended to the first block only.
fn apply_frontmatter_set(
    content: &str,
    block_starts: &[usize],
    key: &str,
    value: &str,
) -> Result<String> {
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let prefix = format!("{key}:");
    let mut out = String::with_capacity(content.len() + key.len() + value.len() + 4);
    let mut next = 0;
    let mut found = false;

    for (n, &start) in block_starts.iter().enumerate() {
        let open = start.saturating_sub(1);
        let delimiter_error = |what: &str| MddError::NoMatch {
            pattern: format!("frontmatter {what} delimiter not found near line {start}"),
        };
        if open < next || lines.get(open).is_none_or(|l| l.trim_end() != "---") {
            return Err(delimiter_error("opening"));
        }
        let close = (open + 1..lines.len())
            .find(|&i| matches!(lines[i].trim_end(), "---" | "..."))
            .ok_or_else(|| delimiter_error("closing"))?;

        out.extend(lines[next..=open].iter().copied());
        for line in &lines[open + 1..close] {
            if line.starts_with(&prefix) {
                out.push_str(&format_line(key, value));
                out.push_str(line_ending(line));
                found = true;
            } else {
                out.push_str(line);
            }
        }
        if n == 0 && !found {
            out.push_str(&format_line(key, value));
            out.push_str(line_ending(lines[open]));
            found = true;
        }
        out.push_str(lines[close]);
        next = close + 1;
    }

    out.extend(lines[next..].iter().copied());
    Ok(out)
}

fn line_ending(line: &str) -> &'static str {
    if line.ends_with("\r\n") {
        "\r\n"
    } else if line.ends_with('\n') {
        "\n"
    } else {
        ""
    }
}

fn format_line(key: &str, value: &str) -> String {
    format!("{key}: {value}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_when_the_opening_delimiter_is_not_where_the_parser_said() {
        let content = "# Title\n\n---\ntype: a\n---\n";
        assert!(apply_frontmatter_set(content, &[1], "type", "b").is_err());
        assert!(apply_frontmatter_set(content, &[99], "type", "b").is_err());
    }

    #[test]
    fn refuses_when_the_closing_delimiter_is_missing() {
        assert!(apply_frontmatter_set("---\ntype: a\n", &[1], "type", "b").is_err());
    }
}
