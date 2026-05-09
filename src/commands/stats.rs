use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(_dir: PathBuf, _cli: &Cli) -> Result<()> {
    eprintln!("mdd stats: not yet implemented");
    Ok(())
}
