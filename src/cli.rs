use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::query::filter::Filter;

/// mdd -- Markdown-Driven Development
///
/// Structural queries on Markdown files for coding agents.
/// Gives agents a compressed map of documentation so they can decide
/// WHAT to read before consuming context.
#[derive(Parser, Debug)]
#[command(name = "mdd", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output format: human (colored), agent (provenance+signature), json
    #[arg(short, long, global = true)]
    pub format: Option<OutputFormat>,

    /// Fail on first malformed file instead of skipping with a warning
    #[arg(long, global = true)]
    pub strict: bool,

    /// Include AGENTS.md, CLAUDE.md, CLAUDE.local.md and README.md in directory scans
    #[arg(long, global = true)]
    pub all_files: bool,
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| "expected a date as YYYY-MM-DD".to_owned())
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Table of contents with offset/limit annotations.
    /// Works on single files and directories.
    Toc(TocArgs),

    /// Extract section content by heading match.
    /// When path is a directory, searches across all .md files.
    Sec(SecArgs),

    /// Find the section containing a given line number.
    /// Prints breadcrumb path to stderr.
    At(AtArgs),

    /// Compressed structural overview for agent orientation.
    /// Shows hot files, directory structure, frontmatter stats, link graph.
    Map(MapArgs),

    /// Cross-file structural query with composable filters.
    Find(FindArgs),

    /// Frontmatter inspection (read-only).
    Fm(FmArgs),

    /// Link graph extraction and validation.
    Links(LinksArgs),

    /// Aggregate matching sections into a single context-efficient output.
    Agg(AggArgs),

    /// List/filter entries in multi-frontmatter documents.
    Parts(PartsArgs),

    /// Directory statistics: file counts, frontmatter coverage, types.
    Stats(StatsArgs),
}

#[derive(clap::Args, Debug)]
pub struct TocArgs {
    /// File or directory path
    pub path: PathBuf,

    /// Optional pattern to filter headings (substring, case-insensitive)
    pub pattern: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct SecArgs {
    /// File or directory path
    pub path: PathBuf,

    /// Heading patterns to match (substring, case-insensitive)
    #[arg(num_args = 1..)]
    pub patterns: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct AtArgs {
    /// File path
    pub path: PathBuf,

    /// Line number (1-based)
    pub line: usize,

    /// Levels to climb up from innermost section (0 = innermost)
    #[arg(default_value = "0")]
    pub level: usize,
}

#[derive(clap::Args, Debug)]
pub struct MapArgs {
    /// Directory path
    pub path: PathBuf,
}

#[derive(clap::Args, Debug)]
pub struct FindArgs {
    /// Directory to search
    pub dir: PathBuf,

    /// Filter by frontmatter 'type' field value
    #[arg(long, short = 't')]
    pub r#type: Option<String>,

    /// Filter by frontmatter 'topic' field value
    #[arg(long)]
    pub topic: Option<String>,

    /// Filter by heading pattern (substring match)
    #[arg(long, short = 'H')]
    pub heading: Option<String>,

    /// Only files containing at least one table
    #[arg(long)]
    pub has_table: bool,

    /// Only files containing code blocks in the specified language
    #[arg(long)]
    pub has_code: Option<String>,

    /// Only files modified after this date (YYYY-MM-DD)
    #[arg(long, value_parser = parse_date)]
    pub newer_than: Option<NaiveDate>,
}

impl FindArgs {
    pub fn filter(&self) -> Filter {
        Filter::new(
            self.r#type.as_deref(),
            self.topic.as_deref(),
            self.heading.as_deref(),
            self.has_table,
            self.has_code.as_deref(),
        )
    }
}

#[derive(clap::Args, Debug)]
pub struct FmArgs {
    /// Directory or file path
    pub path: PathBuf,

    /// List all unique values for a frontmatter field
    #[arg(long)]
    pub list_values: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct LinksArgs {
    /// File or directory path
    pub path: PathBuf,

    /// Check all links for validity (resolve paths)
    #[arg(long)]
    pub check: bool,

    /// Show only broken links
    #[arg(long)]
    pub broken: bool,
}

#[derive(clap::Args, Debug)]
pub struct AggArgs {
    /// Directory to search
    pub dir: PathBuf,

    /// Filter by heading pattern
    #[arg(long, short = 'H')]
    pub heading: Option<String>,

    /// Filter by frontmatter 'type' field
    #[arg(long, short = 't')]
    pub r#type: Option<String>,

    /// Filter by frontmatter 'topic' field
    #[arg(long)]
    pub topic: Option<String>,

    /// Maximum output lines (default: 500)
    #[arg(long, default_value = "500")]
    pub budget: usize,
}

impl AggArgs {
    pub fn filter(&self) -> Filter {
        Filter::new(
            self.r#type.as_deref(),
            self.topic.as_deref(),
            self.heading.as_deref(),
            false,
            None,
        )
    }
}

#[derive(clap::Args, Debug)]
pub struct PartsArgs {
    /// File path
    pub path: PathBuf,

    /// Filter entries by frontmatter field (key=value)
    #[arg(long)]
    pub filter: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct StatsArgs {
    /// Directory path
    pub dir: PathBuf,
}

/// Output format selection.
/// When not specified: `human` on TTY, `agent` on pipe.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    /// Colored, indented, human-readable
    Human,
    /// Context-efficient with provenance: `[file:offset,limit] heading {signature}`
    #[value(help = "Context-efficient with provenance: [file:offset,limit] heading {signature}")]
    Agent,
    /// Structured JSON
    Json,
}
