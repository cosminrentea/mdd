use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(_path: PathBuf, _filter: Option<String>, _cli: &Cli) -> Result<()> {
    eprintln!("mdd parts: not yet implemented");
    Ok(())
}
