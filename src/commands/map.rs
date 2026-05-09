use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(_path: PathBuf, _cli: &Cli) -> Result<()> {
    eprintln!("mdd map: not yet implemented");
    Ok(())
}
