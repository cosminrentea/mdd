pub mod agg;
pub mod at;
pub mod find;
pub mod fm;
pub mod links;
pub mod map;
pub mod parts;
pub mod sec;
pub mod stats;
pub mod toc;

/// How a command that did not fail ended. `main` maps it to the exit code.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Something was found or the report succeeded: exit 0.
    Found,
    /// Nothing was found: exit 1, with the single stderr line `mdd: no match: <detail>`.
    NothingFound(String),
    /// `links --check` or `--broken` found a broken local target: exit 1, no extra line.
    BrokenLinks,
}
