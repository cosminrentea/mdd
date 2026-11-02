use std::path::PathBuf;
use thiserror::Error;

/// Top-level error type for mdd.
///
/// Uses `thiserror` for ergonomic error variants. `main` prints each error as
/// `mdd: error[<code>]: <message>` (see [`MddError::code`]).
#[derive(Error, Debug)]
pub enum MddError {
    #[error("file not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("failed to read file: {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("line {line} is out of range (file has {total_lines} lines)")]
    LineOutOfRange { line: usize, total_lines: usize },

    #[error("not a directory: {path}")]
    NotADirectory { path: PathBuf },

    /// A frontmatter block that is not a YAML mapping. By default only a warning.
    #[error("{path}:{line}: invalid frontmatter ignored: {reason}")]
    InvalidFrontmatter {
        path: PathBuf,
        line: usize,
        reason: String,
    },

    /// A file or walk entry that cannot be read during a directory scan. By default only a warning.
    #[error("skipping {path}: {reason}")]
    Skipped { path: PathBuf, reason: String },
}

impl MddError {
    /// Stable `snake_case` identifier shown in brackets after `error`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "file_not_found",
            Self::LineOutOfRange { .. } => "line_out_of_range",
            Self::NotADirectory { .. } => "not_a_directory",
            Self::InvalidFrontmatter { .. } => "invalid_frontmatter",
            Self::Io { .. } | Self::Skipped { .. } => "io",
        }
    }
}

impl MddError {
    /// The warning (or `--strict` error) for a file that a directory scan could not read.
    pub fn into_skipped(self) -> Self {
        match self {
            Self::Io { path, source } => Self::Skipped {
                path,
                reason: one_line(&source.to_string()),
            },
            Self::FileNotFound { path } => Self::Skipped {
                path,
                reason: "file not found".to_owned(),
            },
            other => other,
        }
    }
}

/// Joins the lines of `text` with single spaces: warnings are exactly one stderr line.
pub fn one_line(text: &str) -> String {
    text.lines().map(str::trim).collect::<Vec<_>>().join(" ")
}

/// Convenience type alias used throughout the codebase.
pub type Result<T> = std::result::Result<T, MddError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn every_variant_has_a_snake_case_code() {
        let path = PathBuf::from("p");
        let io = || std::io::Error::other("boom");
        let cases = [
            (
                MddError::FileNotFound { path: path.clone() },
                "file_not_found",
            ),
            (
                MddError::Io {
                    path: path.clone(),
                    source: io(),
                },
                "io",
            ),
            (
                MddError::LineOutOfRange {
                    line: 1,
                    total_lines: 0,
                },
                "line_out_of_range",
            ),
            (
                MddError::NotADirectory { path: path.clone() },
                "not_a_directory",
            ),
            (
                MddError::InvalidFrontmatter {
                    path: path.clone(),
                    line: 3,
                    reason: "r".to_owned(),
                },
                "invalid_frontmatter",
            ),
            (
                MddError::Skipped {
                    path,
                    reason: "r".to_owned(),
                },
                "io",
            ),
        ];
        for (err, code) in cases {
            assert_eq!(err.code(), code);
        }
    }

    #[test]
    fn io_error_exposes_the_os_error_as_its_source() {
        let err = MddError::Io {
            path: PathBuf::from("p"),
            source: std::io::Error::other("boom"),
        };
        assert_eq!(err.to_string(), "failed to read file: p");
        assert_eq!(err.source().unwrap().to_string(), "boom");
    }

    #[test]
    fn problems_display_as_the_text_after_the_warning_prefix() {
        let frontmatter = MddError::InvalidFrontmatter {
            path: PathBuf::from("d/a.md"),
            line: 4,
            reason: "bad".to_owned(),
        };
        assert_eq!(
            frontmatter.to_string(),
            "d/a.md:4: invalid frontmatter ignored: bad"
        );
        let skipped = MddError::Io {
            path: PathBuf::from("d/b.md"),
            source: std::io::Error::other("boom"),
        }
        .into_skipped();
        assert_eq!(skipped.to_string(), "skipping d/b.md: boom");
        assert!(skipped.source().is_none());
    }

    #[test]
    fn one_line_joins_lines_with_single_spaces() {
        assert_eq!(one_line("a\n  b \nc"), "a b c");
        assert_eq!(one_line("single"), "single");
    }
}
