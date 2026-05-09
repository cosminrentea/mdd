use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(
    _path: PathBuf,
    _list_values: Option<String>,
    _set: Option<String>,
    _apply: bool,
    _cli: &Cli,
) -> Result<()> {
    eprintln!("mdd fm: not yet implemented");
    Ok(())
}
