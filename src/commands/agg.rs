use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::Result;

pub fn run(
    _dir: PathBuf,
    _heading: Option<String>,
    _type_filter: Option<String>,
    _topic: Option<String>,
    _budget: usize,
    _cli: &Cli,
) -> Result<()> {
    eprintln!("mdd agg: not yet implemented");
    Ok(())
}
