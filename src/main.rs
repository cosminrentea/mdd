use clap::Parser;

mod cli;
mod commands;
mod error;
mod output;
mod parse;
mod query;

use cli::{Cli, Command};

fn main() {
    #[cfg(unix)]
    // SAFETY: restoring the default SIGPIPE disposition before any threads start.
    #[allow(unsafe_code)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    // Parse CLI arguments. Clap handles --help, --version, and validation
    // automatically from the derive macros in cli.rs.
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

    // Handle errors with miette's fancy reporting.
    // miette::Report wraps our error and adds the fancy terminal formatting
    // (colors, source snippets, suggestions) defined in the error variants.
    if let Err(err) = result {
        eprintln!("{:?}", miette::Report::new(err));
        std::process::exit(1);
    }
}
