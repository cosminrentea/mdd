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
        strict: _,
    } = Cli::parse();

    let format = output::resolve_format(format);

    let result = match command {
        Command::Toc(args) => commands::toc::run(&args, format),
        Command::Sec(args) => commands::sec::run(&args, format),
        Command::At(args) => commands::at::run(&args, format),
        Command::Map(args) => commands::map::run(&args),
        Command::Find(args) => commands::find::run(&args, format),
        Command::Fm(args) => commands::fm::run(&args),
        Command::Links(args) => commands::links::run(&args, format),
        Command::Agg(args) => commands::agg::run(&args, format),
        Command::Parts(args) => commands::parts::run(&args, format),
        Command::Stats(args) => commands::stats::run(&args),
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
