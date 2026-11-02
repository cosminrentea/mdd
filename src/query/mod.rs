pub mod filter;

use std::path::{Path, PathBuf};

use crate::error::{MddError, Result, one_line};
use crate::parse::MdFile;
use crate::parse::markdown::parse_file;

/// File names (case-sensitive) that a directory walk skips unless `--all-files` is given.
const EXCLUDED_FILE_NAMES: [&str; 4] = ["AGENTS.md", "CLAUDE.local.md", "CLAUDE.md", "README.md"];

/// Settings that every command shares (the global command line flags).
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// A problem with one file or one frontmatter block is an error instead of a warning.
    pub strict: bool,
    /// Directory scans also return AGENTS.md, CLAUDE.md, CLAUDE.local.md and README.md.
    pub all_files: bool,
}

/// Reports one problem: a `mdd: warning: ...` line on stderr, or, with `--strict`, the error.
fn report(problem: MddError, opts: Options) -> Result<()> {
    if opts.strict {
        return Err(problem);
    }
    eprintln!("mdd: warning: {problem}");
    Ok(())
}

/// Reports the frontmatter blocks the parser dropped from `md`.
fn report_frontmatter(md: &MdFile, opts: Options) -> Result<()> {
    for invalid in &md.invalid_frontmatter {
        report(
            MddError::InvalidFrontmatter {
                path: md.path.clone(),
                line: invalid.line,
                reason: invalid.reason.clone(),
            },
            opts,
        )?;
    }
    Ok(())
}

/// Parse one file named on the command line. A read failure is an error;
/// a malformed frontmatter block is reported (see [`Options`]).
pub fn parse_checked(path: &Path, opts: Options) -> Result<MdFile> {
    let md = parse_file(path)?;
    report_frontmatter(&md, opts)?;
    Ok(md)
}

/// One result of walking a directory: a Markdown file or an entry that could not be read.
enum Walked {
    File(PathBuf),
    Unreadable(MddError),
}

impl Walked {
    fn path(&self) -> &Path {
        match self {
            Self::File(path) | Self::Unreadable(MddError::Skipped { path, .. }) => path,
            Self::Unreadable(_) => Path::new(""),
        }
    }
}

/// The innermost error text, normally the OS error: the `ignore` and `walkdir`
/// crates wrap it in prose that repeats the path.
fn reason(err: &ignore::Error) -> String {
    let Some(io) = err.io_error() else {
        return one_line(&err.to_string());
    };
    let mut cause: &(dyn std::error::Error + 'static) = io;
    while let Some(inner) = cause.source() {
        cause = inner;
    }
    one_line(&cause.to_string())
}

fn walk_problem(err: &ignore::Error, fallback: &Path) -> MddError {
    match err {
        ignore::Error::WithDepth { err, .. } => walk_problem(err, fallback),
        ignore::Error::WithPath { path, err } => MddError::Skipped {
            path: path.clone(),
            reason: reason(err),
        },
        ignore::Error::Loop { child, .. } => MddError::Skipped {
            path: child.clone(),
            reason: "file system loop found".to_owned(),
        },
        other => MddError::Skipped {
            path: fallback.to_owned(),
            reason: reason(other),
        },
    }
}

/// Walk a directory collecting all .md file paths and unreadable entries, sorted
/// by path. Follows symlinks, respects .gitignore, excludes infrastructure files
/// (AGENTS.md, CLAUDE.local.md, CLAUDE.md, README.md) unless `--all-files`.
fn walk_md_files(dir: &Path, opts: Options) -> Vec<Walked> {
    let walker = ignore::WalkBuilder::new(dir).follow_links(true).build();

    let mut entries: Vec<Walked> = walker
        .filter_map(|entry| match entry {
            Ok(e) => {
                let name = e.file_name().to_string_lossy();
                let wanted = e.path().extension().is_some_and(|ext| ext == "md")
                    && (opts.all_files || !EXCLUDED_FILE_NAMES.contains(&name.as_ref()));
                wanted.then(|| Walked::File(e.into_path()))
            }
            Err(err) => Some(Walked::Unreadable(walk_problem(&err, dir))),
        })
        .collect();

    entries.sort_by(|a, b| a.path().cmp(b.path()));
    entries
}

/// The result of scanning a directory.
pub struct Scan {
    /// Number of Markdown files found, including those that could not be read.
    pub walked: usize,
    /// The files that were read and parsed, sorted by path.
    pub parsed: Vec<(PathBuf, MdFile)>,
}

/// Walk a directory and parse each .md file. Every problem (an unreadable
/// entry or file, a malformed frontmatter block) is reported once, in path
/// order, before the caller prints anything; with `--strict` the first one is
/// returned as the error.
pub fn scan(dir: &Path, opts: Options) -> Result<Scan> {
    let mut walked = 0;
    let mut parsed = Vec::new();
    for item in walk_md_files(dir, opts) {
        match item {
            Walked::Unreadable(problem) => report(problem, opts)?,
            Walked::File(path) => {
                walked += 1;
                match parse_file(&path) {
                    Ok(md) => {
                        report_frontmatter(&md, opts)?;
                        parsed.push((path, md));
                    }
                    Err(err) => report(err.into_skipped(), opts)?,
                }
            }
        }
    }
    Ok(Scan { walked, parsed })
}
