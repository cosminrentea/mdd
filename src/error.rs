use miette::Diagnostic;
use std::path::PathBuf;
use thiserror::Error;

/// Top-level error type for mdd.
///
/// Uses `thiserror` for ergonomic error variants and `miette` for
/// rich terminal output (source highlighting, suggestions).
///
/// # Rust concept: Error enums
/// In Rust, errors are typically modeled as enums where each variant
/// represents a distinct failure mode. The `#[error(...)]` attribute
/// from thiserror auto-generates the `Display` impl, and `#[diagnostic]`
/// from miette adds structured error metadata (codes, help text).
#[derive(Error, Diagnostic, Debug)]
pub enum MddError {
    #[error("file not found: {path}")]
    #[diagnostic(code(mdd::file_not_found))]
    FileNotFound { path: PathBuf },

    #[error("failed to read file: {path}")]
    #[diagnostic(code(mdd::io_error))]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("line {line} is out of range (file has {total_lines} lines)")]
    #[diagnostic(code(mdd::line_out_of_range))]
    LineOutOfRange { line: usize, total_lines: usize },

    #[error("no headings found in {path}")]
    #[diagnostic(code(mdd::no_headings))]
    NoHeadings { path: PathBuf },

    #[error("no matching sections for pattern: {pattern}")]
    #[diagnostic(code(mdd::no_match))]
    NoMatch { pattern: String },

    #[error("invalid date format: {value} (expected YYYY-MM-DD)")]
    #[diagnostic(code(mdd::invalid_date))]
    InvalidDate { value: String },
}

/// Convenience type alias used throughout the codebase.
///
/// # Rust concept: Type aliases
/// `Result<T>` here means `Result<T, MddError>` -- saves typing the error
/// type everywhere. This is a common Rust pattern (std::io::Result does the same).
pub type Result<T> = std::result::Result<T, MddError>;
