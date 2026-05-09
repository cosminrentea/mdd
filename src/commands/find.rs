use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

#[allow(clippy::too_many_arguments)]
pub fn run(
    _dir: PathBuf,
    _type_filter: Option<String>,
    _topic: Option<String>,
    _heading: Option<String>,
    _has_table: bool,
    _has_code: Option<String>,
    _newer_than: Option<String>,
    _cli: &Cli,
) -> Result<()> {
    eprintln!("mdd find: not yet implemented");
    Ok(())
}
