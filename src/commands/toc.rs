use super::Outcome;
use super::sec::clean_pattern;
use crate::cli::{OutputFormat, TocArgs};
use crate::error::Result;
use crate::output::{self, TocDirEntry, TocDirOutput, TocFileOutput};
use crate::parse::Section;
use crate::query::{self, Options};

pub fn run(args: &TocArgs, format: OutputFormat, opts: Options) -> Result<Outcome> {
    if args.path.is_dir() {
        run_dir(&args.path, args.pattern.as_deref(), format, opts)
    } else {
        run_file(&args.path, args.pattern.as_deref(), format, opts)
    }
}

fn run_file(
    path: &std::path::Path,
    pattern: Option<&str>,
    format: OutputFormat,
    opts: Options,
) -> Result<Outcome> {
    let md = query::parse_checked(path, opts)?;
    let all_sections: Vec<&Section> = md.entries.iter().flat_map(|e| &e.sections).collect();

    if all_sections.is_empty() {
        return Ok(Outcome::NothingFound(format!(
            "no headings found in {}",
            path.display()
        )));
    }

    let filtered: Vec<&Section> = if let Some(pat) = pattern {
        let pat_lower = clean_pattern(pat).to_lowercase();
        let matches: Vec<&Section> = all_sections
            .into_iter()
            .filter(|s| s.title.to_lowercase().contains(&pat_lower))
            .collect();
        if matches.is_empty() {
            return Ok(no_matching_sections(pat));
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

    Ok(Outcome::Found)
}

fn no_matching_sections(pattern: &str) -> Outcome {
    Outcome::NothingFound(format!("no matching sections for pattern: {pattern}"))
}

fn run_dir(
    dir: &std::path::Path,
    pattern: Option<&str>,
    format: OutputFormat,
    opts: Options,
) -> Result<Outcome> {
    let scan = query::scan(dir, opts)?;

    let pat_lower = pattern.map(|p| clean_pattern(p).to_lowercase());
    let mut dir_entries: Vec<TocDirEntry> = Vec::new();

    for (file_path, md) in &scan.parsed {
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
            byte_size: md.source.len(),
            first_heading,
            sub_headings,
            min_level,
        });
    }

    let total_files = dir_entries.len();
    if total_files == 0
        && let Some(pat) = pattern
    {
        return Ok(no_matching_sections(pat));
    }

    output::render_toc_dir(
        format,
        &TocDirOutput {
            entries: dir_entries,
            total_files,
        },
    );

    if scan.walked == 0 {
        return Ok(Outcome::NothingFound(format!(
            "no Markdown files in {}",
            dir.display()
        )));
    }
    Ok(Outcome::Found)
}
