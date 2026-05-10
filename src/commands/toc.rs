use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::{MddError, Result};
use crate::output::{self, TocDirEntry, TocDirOutput, TocFileOutput};
use crate::parse::markdown::{parse_content, parse_file};
use crate::parse::Section;

pub fn run(path: PathBuf, pattern: Option<String>, cli: &Cli) -> Result<()> {
    if path.is_dir() {
        run_dir(&path, pattern.as_deref(), cli)
    } else {
        run_file(&path, pattern.as_deref(), cli)
    }
}

fn run_file(path: &std::path::Path, pattern: Option<&str>, cli: &Cli) -> Result<()> {
    let md = parse_file(path)?;
    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    if all_sections.is_empty() {
        return Err(MddError::NoHeadings {
            path: path.to_owned(),
        });
    }

    let filtered: Vec<&Section> = if let Some(pat) = pattern {
        let pat_lower = clean_toc_pattern(pat).to_lowercase();
        let matches: Vec<&Section> = all_sections
            .into_iter()
            .filter(|s| s.title.to_lowercase().contains(&pat_lower))
            .collect();
        if matches.is_empty() {
            return Err(MddError::NoMatch {
                pattern: pat.to_owned(),
            });
        }
        matches
    } else {
        all_sections
    };

    let min_level = filtered.iter().map(|s| s.level).min().unwrap_or(1);
    let format = output::resolve_format(&cli.format);

    output::render_toc_file(
        &format,
        &TocFileOutput {
            path,
            sections: filtered,
            min_level,
        },
    );

    Ok(())
}

fn run_dir(dir: &std::path::Path, pattern: Option<&str>, cli: &Cli) -> Result<()> {
    if !dir.exists() {
        return Err(MddError::FileNotFound {
            path: dir.to_owned(),
        });
    }

    let walker = ignore::WalkBuilder::new(dir).follow_links(true).build();

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

    let pat_lower = pattern.map(|p| clean_toc_pattern(p).to_lowercase());
    let mut dir_entries: Vec<TocDirEntry> = Vec::new();

    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let md = parse_content(&content);
        let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

        let first_heading = all_sections
            .first()
            .map(|s| s.title.clone())
            .unwrap_or_else(|| "(no headings)".to_owned());

        let min_level = all_sections.first().map(|s| s.level).unwrap_or(1);
        let sub_headings: Vec<Section> = all_sections
            .iter()
            .skip(1)
            .filter(|s| s.level - min_level <= 1)
            .map(|s| (*s).clone())
            .collect();

        if let Some(ref pat) = pat_lower {
            let rel = file_path
                .strip_prefix(dir)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_lowercase();
            let heading_match = first_heading.to_lowercase().contains(pat.as_str());
            let sub_match = sub_headings
                .iter()
                .any(|s| s.title.to_lowercase().contains(pat.as_str()));
            if !rel.contains(pat.as_str()) && !heading_match && !sub_match {
                continue;
            }
        }

        let rel_path = file_path
            .strip_prefix(dir)
            .unwrap_or(file_path)
            .to_owned();
        dir_entries.push(TocDirEntry {
            rel_path,
            total_lines: md.total_lines,
            byte_size: content.len(),
            first_heading,
            sub_headings,
            min_level,
        });
    }

    let total_files = dir_entries.len();
    if total_files == 0 {
        if let Some(pat) = pattern {
            return Err(MddError::NoMatch {
                pattern: pat.to_owned(),
            });
        }
    }

    let format = output::resolve_format(&cli.format);
    output::render_toc_dir(&format, &TocDirOutput {
        entries: dir_entries,
        total_files,
    });

    Ok(())
}

fn clean_toc_pattern(pat: &str) -> String {
    let mut s = pat.trim().to_owned();
    if let Some(stripped) = s.strip_prefix("- ") {
        s = stripped.to_owned();
    }
    if let Some(idx) = s.rfind(" (offset=") {
        s.truncate(idx);
    }
    s
}
