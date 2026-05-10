use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(_dir: PathBuf, _clear: bool, _cli: &Cli) -> Result<()> {
    eprintln!("mdd cache: not implemented (not needed -- 51ms for 320 files without caching)");
    Ok(())
}
