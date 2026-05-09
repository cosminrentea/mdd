use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(_path: PathBuf, _check: bool, _broken: bool, _cli: &Cli) -> Result<()> {
    eprintln!("mdd links: not yet implemented");
    Ok(())
}
