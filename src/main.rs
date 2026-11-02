use std::error::Error;
use std::process::ExitCode;

use clap::Parser;

mod cli;
mod commands;
mod error;
mod output;
mod parse;
mod query;

use cli::{Cli, Command};
use commands::Outcome;
use error::MddError;
use query::Options;

fn main() -> ExitCode {
    #[cfg(unix)]
    // SAFETY: restoring the default SIGPIPE disposition before any threads start.
    #[allow(unsafe_code)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let Cli {
        command,
        format,
        strict,
        all_files,
    } = Cli::parse();
    let opts = Options { strict, all_files };

    let format = output::resolve_format(format);

    let result = match command {
        Command::Toc(args) => commands::toc::run(&args, format, opts),
        Command::Sec(args) => commands::sec::run(&args, format, opts),
        Command::At(args) => commands::at::run(&args, format, opts),
        Command::Map(args) => commands::map::run(&args, opts),
        Command::Find(args) => commands::find::run(&args, format, opts),
        Command::Fm(args) => commands::fm::run(&args, opts),
        Command::Links(args) => commands::links::run(&args, format, opts),
        Command::Agg(args) => commands::agg::run(&args, format, opts),
        Command::Parts(args) => commands::parts::run(&args, format, opts),
        Command::Stats(args) => commands::stats::run(&args, opts),
    };

    match result {
        Ok(Outcome::Found) => ExitCode::SUCCESS,
        Ok(Outcome::NothingFound(detail)) => {
            eprintln!("mdd: no match: {detail}");
            ExitCode::from(1)
        }
        Ok(Outcome::BrokenLinks) => ExitCode::from(1),
        Err(err) => {
            report(&err);
            ExitCode::from(2)
        }
    }
}

/// Prints `mdd: error[<code>]: <message>` and one `  caused by: <source>` line per source.
fn report(err: &MddError) {
    eprintln!("mdd: error[{}]: {err}", err.code());
    let mut source = err.source();
    while let Some(cause) = source {
        eprintln!("  caused by: {cause}");
        source = cause.source();
    }
}
