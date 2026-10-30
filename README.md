# mdd

[![Crates.io](https://img.shields.io/crates/v/mdd.svg)](https://crates.io/crates/mdd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Updated: 2026-10-31

`mdd` is a Rust CLI for structural queries on Markdown. It helps coding agents and humans inspect document headings, select sections, and search collections without reading every file in full.

## Install

Updated: 2026-10-31

On macOS arm64, install with Homebrew:

```bash
brew install cosminrentea/tap/mdd
```

Or install from the repository with a Rust toolchain:

```bash
cargo install --git https://github.com/cosminrentea/mdd
```

## Quick start

Updated: 2026-10-31

```bash
# Inspect the outline, then read the sections you need.
mdd toc README.md
mdd sec README.md "Install" "Commands"

# Find a section containing a 1-based line number.
mdd at README.md 10

# Explore a collection of Markdown files.
mdd toc docs/
mdd find docs/ --has-code bash
```

By default, output is human-readable on a terminal and uses an agent-oriented format when piped. Use `--format human`, `--format agent`, or `--format json` to select a format explicitly. See the [command reference](docs/commands.md) for format limitations.

## Commands

Updated: 2026-10-31

| Command | Purpose |
| --- | --- |
| `toc` | Outline a file or index a directory. |
| `sec` | Extract sections from a file by heading. |
| `at` | Extract the section containing a line. |
| `find` | Find sections using structural and frontmatter filters. |
| `map` | Summarize a directory. |
| `agg` | Aggregate matching sections within a line budget. |
| `links` | List links and check local targets. |
| `parts` | List or filter entries in a multi-frontmatter file. |
| `fm` | Inspect or update frontmatter. |
| `stats` | Summarize directory statistics. |
| `cache` | Placeholder; caching is not implemented. |

## Documentation

Updated: 2026-10-31

- [Command reference](docs/commands.md)
- [Usage and agent workflows](docs/workflows.md)
- [Architecture and development](docs/architecture.md)

## License

Updated: 2026-10-31

[MIT](LICENSE)
