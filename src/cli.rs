use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Table of contents with offset/limit annotations.
    /// Works on single files and directories.
    Toc {
        /// File or directory path
        path: PathBuf,

        /// Optional pattern to filter headings (substring, case-insensitive)
        pattern: Option<String>,
    },

    /// Extract section content by heading match.
    /// When path is a directory, searches across all .md files.
    Sec {
        /// File or directory path
        path: PathBuf,

        /// Heading pattern to match (substring, case-insensitive)
        pattern: String,
    },

    /// Find the section containing a given line number.
    /// Prints breadcrumb path to stderr.
    At {
        /// File path
        path: PathBuf,

        /// Line number (1-based)
        line: usize,

        /// Levels to climb up from innermost section (0 = innermost)
        #[arg(default_value = "0")]
        level: usize,
    },

    /// Compressed structural overview for agent orientation.
    /// Shows hot files, directory structure, frontmatter stats, link graph.
    Map {
        /// Directory path
        path: PathBuf,
    },

    /// Cross-file structural query with composable filters.
    Find {
        /// Directory to search
        dir: PathBuf,

        /// Filter by frontmatter 'type' field value
        #[arg(long, short = 't')]
        r#type: Option<String>,

        /// Filter by frontmatter 'topic' field value
        #[arg(long)]
        topic: Option<String>,

        /// Filter by heading pattern (substring match)
        #[arg(long, short = 'H')]
        heading: Option<String>,

        /// Only files containing at least one table
        #[arg(long)]
        has_table: bool,

        /// Only files containing code blocks in the specified language
        #[arg(long)]
        has_code: Option<String>,

        /// Only files modified after this date (YYYY-MM-DD)
        #[arg(long)]
        newer_than: Option<String>,
    },

    /// Frontmatter inspection and modification.
    Fm {
        /// Directory or file path
        path: PathBuf,

        /// List all unique values for a frontmatter field
        #[arg(long)]
        list_values: Option<String>,

        /// Set a frontmatter field (key=value format). Dry-run by default.
        #[arg(long)]
        set: Option<String>,

        /// Actually apply the --set modification (without this, only prints what would change)
        #[arg(long)]
        apply: bool,
    },

    /// Link graph extraction and validation.
    Links {
        /// File or directory path
        path: PathBuf,

        /// Check all links for validity (resolve paths)
        #[arg(long)]
        check: bool,

        /// Show only broken links
        #[arg(long)]
        broken: bool,
    },

    /// Aggregate matching sections into a single context-efficient output.
    Agg {
        /// Directory to search
        dir: PathBuf,

        /// Filter by heading pattern
        #[arg(long, short = 'H')]
        heading: Option<String>,

        /// Filter by frontmatter 'type' field
        #[arg(long, short = 't')]
        r#type: Option<String>,

        /// Filter by frontmatter 'topic' field
        #[arg(long)]
        topic: Option<String>,

        /// Maximum output lines (default: 500)
        #[arg(long, default_value = "500")]
        budget: usize,
    },

    /// List/filter entries in multi-frontmatter documents.
    Parts {
        /// File path
        path: PathBuf,

        /// Filter entries by frontmatter field (key=value)
        #[arg(long)]
        filter: Option<String>,
    },

    /// Directory statistics: file counts, frontmatter coverage, types.
    Stats {
        /// Directory path
        dir: PathBuf,
    },

    /// Show or clear the transparent parse cache.
    Cache {
        /// Directory whose cache to inspect
        dir: PathBuf,

        /// Clear the cache for this directory
        #[arg(long)]
        clear: bool,
    },
}

/// Output format selection.
/// When not specified: `human` on TTY, `agent` on pipe.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    /// Colored, indented, human-readable
    Human,
    /// Context-efficient with provenance: [file:offset,limit] heading {signature}
    Agent,
    /// Structured JSON
    Json,
}
