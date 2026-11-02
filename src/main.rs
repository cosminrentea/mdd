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
    let cli = Cli::parse();

    // Match on a reference to cli.command. We clone() where handlers need
    // owned values -- acceptable since these are small CLI argument strings,
    // not large data.
    let format = output::resolve_format(cli.format);

    let result = match &cli.command {
        Command::Toc { path, pattern } => commands::toc::run(path.clone(), pattern.clone(), format),
        Command::Sec { path, patterns } => {
            commands::sec::run(path.clone(), patterns.clone(), format)
        }
        Command::At { path, line, level } => commands::at::run(path.clone(), *line, *level, format),
        Command::Map { path } => commands::map::run(path.clone()),
        Command::Find {
            dir,
            r#type,
            topic,
            heading,
            has_table,
            has_code,
            newer_than,
        } => commands::find::run(
            dir.clone(),
            r#type.clone(),
            topic.clone(),
            heading.clone(),
            *has_table,
            has_code.clone(),
            *newer_than,
            format,
        ),
        Command::Fm {
            path,
            list_values,
            set,
            apply,
        } => commands::fm::run(path.clone(), list_values.clone(), set.clone(), *apply),
        Command::Links {
            path,
            check,
            broken,
        } => commands::links::run(path.clone(), *check, *broken, format),
        Command::Agg {
            dir,
            heading,
            r#type,
            topic,
            budget,
        } => commands::agg::run(
            dir.clone(),
            heading.clone(),
            r#type.clone(),
            topic.clone(),
            *budget,
            format,
        ),
        Command::Parts { path, filter } => {
            commands::parts::run(path.clone(), filter.clone(), format)
        }
        Command::Stats { dir } => commands::stats::run(dir.clone()),
    };

    // Handle errors with miette's fancy reporting.
    // miette::Report wraps our error and adds the fancy terminal formatting
    // (colors, source snippets, suggestions) defined in the error variants.
    if let Err(err) = result {
        eprintln!("{:?}", miette::Report::new(err));
        std::process::exit(1);
    }
}
