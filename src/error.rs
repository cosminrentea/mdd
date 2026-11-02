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
}

impl MddError {
    /// Stable `snake_case` identifier shown in brackets after `error`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "file_not_found",
            Self::Io { .. } => "io",
            Self::LineOutOfRange { .. } => "line_out_of_range",
            Self::NotADirectory { .. } => "not_a_directory",
        }
    }
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
            (MddError::NotADirectory { path }, "not_a_directory"),
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
}
