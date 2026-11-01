# Command reference

Updated: 2026-10-31

Run `mdd --help` or `mdd <command> --help` for the installed version's options. This reference describes the repository's 0.2.0 source.

## Global options and output

Updated: 2026-10-31

- `-f, --format human|agent|json` selects an output format. Without it, `mdd` selects `human` for a terminal and `agent` otherwise.
- `-h, --help` and `-V, --version` print help and version information.
- `--strict` is accepted, but the current implementation does not use it to change error handling. Directory walkers can silently skip unreadable files.

For supported commands, agent output can contain `[file:offset,limit]`: `offset` is a 1-based line number and `limit` is a line count. Signatures can include table, code, list, word, and link counts. JSON shape varies by command: `find`, `parts`, and `links` emit one JSON object per line, while `agg` emits an array. `map`, `fm`, `stats`, and `cache` do not honor `--format`.

## Outline and extract

Updated: 2026-10-31

- `mdd toc <path> [pattern]` outlines a file or indexes a directory. The optional pattern matches headings case-insensitively; for directories it also matches relative paths. A file with no headings or no matching headings produces an error.
- `mdd sec <file> <pattern>...` prints all sections whose headings contain each pattern, case-insensitively. A parent section includes nested sections. Multiple patterns can produce duplicate content. An unmatched pattern warns if others match; no matches is an error. Despite the help text, directory input is not supported by this implementation.
- `mdd at <file> <line> [level]` prints the innermost section containing a 1-based line. `level` climbs to enclosing headings, capped at the outermost heading. A line before the first heading prints a message rather than a section.

## Query a directory

Updated: 2026-10-31

- `mdd find <dir> [filters]` finds matching sections. Combine `-t, --type <value>`, `--topic <value>`, `-H, --heading <pattern>`, `--has-table`, `--has-code <language>`, and `--newer-than <YYYY-MM-DD>`. Filters combine with AND. Type matches exactly, topic and heading match substrings, all case-insensitively. The date uses the file's modification time, compared inclusively at UTC midnight. An invalid date is a usage error (exit code 2).
- `mdd agg <dir> [-H <pattern>] [-t <type>] [--topic <topic>] [--budget <lines>]` prints matching sections, newest files first, then shorter sections first. The default budget is 500 content lines. The first matching section is emitted even if it exceeds the budget; subsequent sections stop at the first one that would exceed it.
- `mdd map <dir>` summarizes recent files, directory structure, topics, and content features. Recent means modified in the last three days.
- `mdd stats <dir>` reports file and line counts, frontmatter coverage, types, content features, and heading depth.

These directory commands and directory `toc` use ignore-aware traversal, follow symlinks, and exclude files named `README.md`, `AGENTS.md`, `CLAUDE.md`, and `CLAUDE.local.md`. Target a README directly with `toc` or `sec` when needed.

## Links and frontmatter

Updated: 2026-10-31

- `mdd links <path> [--check | --broken]` lists Markdown links. `--check` tests whether local target paths exist; `--broken` implies checking and prints only broken local targets. HTTP, HTTPS, and mail links are labeled external, not fetched. Anchor fragments are not validated, and broken targets do not make the command fail. Directory scans use the exclusions above.
- `mdd parts <file> [--filter key=value]` lists entries in a multi-frontmatter document. The filter compares the selected frontmatter value case-insensitively.
- `mdd fm <path>` prints frontmatter fields for a file or coverage for a directory. `--list-values <field>` lists unique values. `--set key=value` previews an edit to a single file; add `--apply` to write it. Only the frontmatter blocks are rewritten, line endings are preserved, and a directory path is an error (exit code 1). Directory `fm` scans do not use the filename exclusions above.
- `mdd cache <dir> [--clear]` prints a not-implemented message; it does not inspect or clear a cache.
