pub mod filter;
pub mod heading;
pub mod predicates;

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::NaiveDate;

use crate::error::{MddError, Result};
use crate::parse::markdown::parse_file;
use crate::parse::MdFile;

/// Walk a directory collecting all .md file paths, sorted alphabetically.
/// Follows symlinks, respects .gitignore, excludes infrastructure files
/// (AGENTS.md, CLAUDE.local.md, CLAUDE.md, README.md).
pub fn walk_md_files(dir: &Path) -> Vec<PathBuf> {
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
    files
}

/// Walk a directory, parse each .md file, and return (path, parsed) pairs.
/// Files that fail to parse are silently skipped.
pub fn walk_and_parse(dir: &Path) -> Vec<(PathBuf, MdFile)> {
    walk_md_files(dir)
        .into_iter()
        .filter_map(|p| parse_file(&p).ok().map(|md| (p, md)))
        .collect()
}

/// Parse a date string (YYYY-MM-DD) into a SystemTime for mtime comparisons.
pub fn parse_date_to_system_time(date_str: &str) -> Result<SystemTime> {
    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| MddError::InvalidDate {
            value: format!("'{}': {}", date_str, e),
        })?;
    let datetime = date.and_hms_opt(0, 0, 0).unwrap();
    let timestamp = datetime.and_utc().timestamp();
    Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64))
}
