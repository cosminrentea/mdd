use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(_path: PathBuf, _pattern: String, _cli: &Cli) -> Result<()> {
    eprintln!("mdd sec: not yet implemented");
    Ok(())
}
