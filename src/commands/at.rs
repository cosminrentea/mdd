use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(_path: PathBuf, _line: usize, _level: usize, _cli: &Cli) -> Result<()> {
    eprintln!("mdd at: not yet implemented");
    Ok(())
}
