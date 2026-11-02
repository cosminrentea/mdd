use crate::cli::{OutputFormat, TocArgs};
use crate::error::{MddError, Result};
use crate::output::{self, TocDirEntry, TocDirOutput, TocFileOutput};
use crate::parse::Section;
use crate::parse::markdown::{parse_content, parse_file};
use crate::query::walk_md_files;

pub fn run(args: &TocArgs, format: OutputFormat) -> Result<()> {
    if args.path.is_dir() {
        run_dir(&args.path, args.pattern.as_deref(), format)
    } else {
        run_file(&args.path, args.pattern.as_deref(), format)
    }
}

fn run_file(path: &std::path::Path, pattern: Option<&str>, format: OutputFormat) -> Result<()> {
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

    output::render_toc_file(
        format,
        &TocFileOutput {
            path,
            sections: filtered,
            min_level,
        },
    );

    Ok(())
}

fn run_dir(dir: &std::path::Path, pattern: Option<&str>, format: OutputFormat) -> Result<()> {
    if !dir.exists() {
        return Err(MddError::FileNotFound {
            path: dir.to_owned(),
        });
    }

    let files = walk_md_files(dir);

    let pat_lower = pattern.map(|p| clean_toc_pattern(p).to_lowercase());
    let mut dir_entries: Vec<TocDirEntry> = Vec::new();

    for file_path in &files {
        let Ok(content) = std::fs::read_to_string(file_path) else {
            continue;
        };

        let md = parse_content(&content);
        let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

        let first_heading = all_sections
            .first()
            .map_or_else(|| "(no headings)".to_owned(), |s| s.title.clone());

        let min_level = all_sections.iter().map(|s| s.level).min().unwrap_or(1);
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

        let rel_path = file_path.strip_prefix(dir).unwrap_or(file_path).to_owned();
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
    if total_files == 0
        && let Some(pat) = pattern
    {
        return Err(MddError::NoMatch {
            pattern: pat.to_owned(),
        });
    }

    output::render_toc_dir(
        format,
        &TocDirOutput {
            entries: dir_entries,
            total_files,
        },
    );

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
