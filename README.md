# mdd

[![Crates.io](https://img.shields.io/crates/v/mdd.svg)](https://crates.io/crates/mdd)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Structural queries on Markdown files for coding agents.

`mdd` gives AI coding agents (and humans) a compressed map of documentation so they can decide **what** to read before consuming context tokens. Think of it as `jq` for Markdown structure.

## Features

- **`toc`** -- Table of contents with byte-offset annotations for precise retrieval
- **`sec`** -- Extract one or more sections by heading pattern (substring, case-insensitive)
- **`at`** -- Find the section containing a given line number (great after grep)
- **`find`** -- Cross-file structural queries with composable filters (type, topic, tables, code)
- **`map`** -- Compressed directory overview: hot files, topic distribution, content stats
- **`agg`** -- Aggregate matching sections across files into budget-constrained output
- **`links`** -- Link graph extraction and broken-link detection
- **`parts`** -- List/filter entries in multi-frontmatter documents
- **`fm`** -- Frontmatter inspection and modification
- **`stats`** -- Directory statistics: file counts, frontmatter coverage, types

## Installation

### Homebrew (macOS arm64)

```bash
brew install cosminrentea/tap/mdd
```

### Cargo (any platform with Rust toolchain)

```bash
cargo install --git https://github.com/cosminrentea/mdd
```

## Quick Start

```bash
# See the structure of a large Markdown file
mdd toc docs/architecture.md

# Extract specific sections (one or more patterns)
mdd sec docs/architecture.md "Deployment" "Security"

# After grep finds a match, get the full surrounding section
mdd at docs/architecture.md 142

# Find all files with tables in a directory
mdd find docs/ --has-table

# Get a compressed overview of a documentation directory
mdd map docs/
```

## Commands

| Command | Description |
|---------|-------------|
| `toc <path> [pattern]` | Table of contents with offset/limit. Works on files and directories. |
| `sec <path> <pattern...>` | Extract section content by heading match (one or more patterns). |
| `at <path> <line> [level]` | Find section containing a line number. Optional level to climb up. |
| `find <dir> [filters]` | Cross-file query: `--type`, `--topic`, `--heading`, `--has-table`, `--has-code`, `--newer-than` |
| `map <dir>` | Compressed structural overview for agent orientation. |
| `agg <dir> [filters]` | Aggregate sections across files with `--budget` line limit. |
| `links <path>` | Link listing. Add `--check` to validate, `--broken` for broken only. |
| `parts <path>` | List entries in multi-frontmatter documents. Filter with `--filter key=value`. |
| `fm <path>` | Frontmatter tools: `--list-values <field>`, `--set key=value --apply`. |
| `stats <dir>` | File counts, frontmatter coverage, content feature distribution. |
| `cache <dir>` | Inspect or `--clear` the transparent parse cache. |

## Output Formats

Use `-f <format>` to control output:

| Format | When to use |
|--------|-------------|
| `human` | Interactive terminal use. Colored, indented. Default on TTY. |
| `agent` | AI agent consumption. Provenance lines `[file:offset,limit]` for precise retrieval. Default when piped. |
| `json` | Programmatic use. Structured, parseable by `jq`. |

## For Developers

### Prerequisites

- Rust 1.70+ (stable)

### Building and Testing

```bash
cargo build
cargo test
cargo clippy
```

### Architecture

```
src/
  cli.rs          -- Clap derive-based CLI definition
  main.rs         -- Entry point, command dispatch
  error.rs        -- Error types (thiserror + miette)
  commands/       -- One module per subcommand
  parse/          -- Markdown parser (pulldown-cmark), data model, features
  output/         -- Rendering: human (colored), agent (provenance), json
  query/          -- Filter predicates for find/agg
tests/
  cli_smoke.rs    -- Integration tests (assert_cmd)
  proptest_parser.rs -- Property-based parser fuzzing
  snapshots_json.rs  -- JSON output snapshot tests (insta)
```

Key design decisions:
- Single streaming pass over each file via `pulldown-cmark` offset iterator
- Byte offsets converted to 1-based line numbers using pre-computed lookup table
- Sections include `offset` and `limit` designed for direct use with editor/tool `Read(offset, limit)` calls
- Parent hierarchy tracked via `parent_idx` for breadcrumb navigation
- Feature accumulation (tables, code blocks, lists, word count) happens during parse, not as a second pass

### Adding a new command

1. Add variant to `Command` enum in `src/cli.rs`
2. Create handler in `src/commands/<name>.rs`, register in `src/commands/mod.rs`
3. Add dispatch arm in `src/main.rs`
4. Add output types in `src/output/mod.rs` if needed, implement for all three formats
5. Add integration tests in `tests/cli_smoke.rs`

## License

MIT
