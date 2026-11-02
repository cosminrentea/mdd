use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output;
use crate::parse::Link;
use crate::parse::markdown::parse_file;
use crate::query;

pub fn run(path: PathBuf, check: bool, broken: bool, cli: &Cli) -> Result<()> {
    if !path.exists() {
        return Err(MddError::FileNotFound { path });
    }

    let format = output::resolve_format(cli.format.as_ref());
    let check = check || broken;

    let all_links: Vec<(PathBuf, Vec<Link>)> = if path.is_dir() {
        query::walk_and_parse(&path)
            .into_iter()
            .filter(|(_, md)| !md.links.is_empty())
            .map(|(p, md)| {
                let rel = p.strip_prefix(&path).unwrap_or(&p).to_path_buf();
                (rel, md.links)
            })
            .collect()
    } else {
        let md = parse_file(&path)?;
        let rel = path.file_name().map_or_else(|| path.clone(), PathBuf::from);
        vec![(rel, md.links)]
    };

    let mut total = 0usize;
    let mut broken_count = 0usize;

    for (rel_path, links) in &all_links {
        for link in links {
            total += 1;
            let status = if check {
                match &link.resolved_path {
                    None => LinkStatus::External,
                    Some(resolved) => {
                        if resolved.exists() {
                            LinkStatus::Ok
                        } else {
                            broken_count += 1;
                            LinkStatus::Broken
                        }
                    }
                }
            } else {
                LinkStatus::Unchecked
            };

            if broken && !matches!(status, LinkStatus::Broken) {
                continue;
            }

            let kind_str = "Markdown";

            let status_str = match status {
                LinkStatus::Ok => "OK",
                LinkStatus::Broken => "BROKEN",
                LinkStatus::External => "External",
                LinkStatus::Unchecked => "",
            };

            match &format {
                crate::cli::OutputFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "file": rel_path.display().to_string(),
                            "line": link.line,
                            "target": link.target,
                            "kind": kind_str,
                            "status": status_str,
                        })
                    );
                }
                crate::cli::OutputFormat::Agent | crate::cli::OutputFormat::Human => {
                    let status_suffix = if status_str.is_empty() {
                        String::new()
                    } else {
                        format!(", {status_str}")
                    };
                    println!(
                        "[{}:{}] -> {} ({}{})",
                        rel_path.display(),
                        link.line,
                        link.target,
                        kind_str,
                        status_suffix,
                    );
                }
            }
        }
    }

    if check {
        eprintln!("\n{total} links total, {broken_count} broken");
    } else {
        eprintln!("\n{total} links total");
    }

    Ok(())
}

enum LinkStatus {
    Ok,
    Broken,
    External,
    Unchecked,
}
