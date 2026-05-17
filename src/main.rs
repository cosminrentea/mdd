use clap::Parser;
use mdd::cli::{Cli, Command};
use mdd::commands;

fn main() {
    // Parse CLI arguments. Clap handles --help, --version, and validation
    // automatically from the derive macros in cli.rs.
    let cli = Cli::parse();

    // Match on a reference to cli.command. This avoids moving fields out of
    // the Cli struct, so we can still pass &cli to each command handler.
    //
    // Rust concept: when you match on `&enum_value`, destructured bindings
    // are automatically references (&PathBuf, &Option<String>, etc.).
    // We clone() where handlers need owned values -- acceptable since these
    // are small CLI argument strings, not large data.
    let result = match &cli.command {
        Command::Toc { path, pattern } => commands::toc::run(path.clone(), pattern.clone(), &cli),
        Command::Sec { path, patterns } => commands::sec::run(path.clone(), patterns.clone(), &cli),
        Command::At { path, line, level } => commands::at::run(path.clone(), *line, *level, &cli),
        Command::Map { path } => commands::map::run(path.clone(), &cli),
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
            newer_than.clone(),
            &cli,
        ),
        Command::Fm {
            path,
            list_values,
            set,
            apply,
        } => commands::fm::run(path.clone(), list_values.clone(), set.clone(), *apply, &cli),
        Command::Links {
            path,
            check,
            broken,
        } => commands::links::run(path.clone(), *check, *broken, &cli),
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
            &cli,
        ),
        Command::Parts { path, filter } => commands::parts::run(path.clone(), filter.clone(), &cli),
        Command::Stats { dir } => commands::stats::run(dir.clone(), &cli),
        Command::Cache { dir, clear } => commands::cache::run(dir.clone(), *clear, &cli),
    };

    // Handle errors with miette's fancy reporting.
    // miette::Report wraps our error and adds the fancy terminal formatting
    // (colors, source snippets, suggestions) defined in the error variants.
    if let Err(err) = result {
        eprintln!("{:?}", miette::Report::new(err));
        std::process::exit(1);
    }
}
