# Command reference

Updated: 2026-11-02

Run `mdd --help` or `mdd <command> --help` for the installed version's options. This reference describes the repository's 0.2.1 source.

## Global options and output

Updated: 2026-11-02

- `-f, --format human|agent|json` selects an output format. Without it, `mdd` selects `human` for a terminal and `agent` otherwise.
- `-h, --help` and `-V, --version` print help and version information.
- `--all-files` includes `AGENTS.md`, `CLAUDE.md`, `CLAUDE.local.md`, and `README.md` in directory scans; without it they are skipped (see below).
- `--strict` makes problems with a single file or frontmatter block errors. Without it, a frontmatter block that is not a YAML mapping prints `mdd: warning: <path>:<line>: invalid frontmatter ignored: <reason>` (its fields are dropped), and a file or directory entry that a directory scan cannot read prints `mdd: warning: skipping <path>: <reason>`; processing and the exit code are unaffected. An empty frontmatter block is not a problem. With `--strict` the first such problem, in file order, is an error (`invalid_frontmatter` or `io`, exit 2) and nothing is printed to stdout.

### Exit codes and errors

Updated: 2026-11-02

Exit codes follow `grep`: `0` means something was found or the report succeeded, `1` means nothing was found, and `2` means an error (usage errors from the argument parser also exit 2). When nothing is found, stderr holds exactly one line, `mdd: no match: <detail>`, and stdout is empty (except `map`, `fm <dir>` and `toc <dir>` on a directory without Markdown files, which still print their empty report). `links --check` and `links --broken` also exit 1 when at least one local target is broken; their totals line is unchanged and no extra line is printed. Errors print `mdd: error[<code>]: <message>` plus one `  caused by: <source>` line per underlying error, without colors or blank lines. Codes: `file_not_found`, `io`, `line_out_of_range`, `not_a_directory`, `invalid_frontmatter` (only with `--strict`). A missing path is `file_not_found`; a path that cannot be read, such as a directory given to `sec`, is `io`; `map` and `stats` given a file are `not_a_directory`.

For supported commands, agent output can contain `[file:offset,limit]`: `offset` is a 1-based line number and `limit` is a line count. Signatures can include table, code, list, word, and link counts. JSON shape varies by command: `find`, `parts`, and `links` emit one JSON object per line, while `agg` emits an array. `map`, `fm`, and `stats` do not honor `--format`.

## Outline and extract

Updated: 2026-11-02

- `mdd toc <path> [pattern]` outlines a file or indexes a directory. The optional pattern matches headings case-insensitively; for directories it also matches relative paths. A file with no headings, or no matching headings, exits 1 (nothing found).
- `mdd sec <file> <pattern>...` prints all sections whose headings contain each pattern, case-insensitively. A parent section includes nested sections. Multiple patterns can produce duplicate content. An unmatched pattern warns if others match; no matches exits 1. Despite the help text, directory input is not supported by this implementation.
- `mdd at <file> <line> [level]` prints the innermost section containing a 1-based line. `level` climbs to enclosing headings, capped at the outermost heading. A line before the first heading exits 1 (nothing found); a line past the end of the file is an error.

## Query a directory

Updated: 2026-11-02

- `mdd find <dir> [filters]` finds matching sections. Combine `-t, --type <value>`, `--topic <value>`, `-H, --heading <pattern>`, `--has-table`, `--has-code <language>`, and `--newer-than <YYYY-MM-DD>`. Filters combine with AND. Type matches exactly, topic and heading match substrings, all case-insensitively. The date uses the file's modification time, compared inclusively at UTC midnight. An invalid date is a usage error (exit code 2).
- `mdd agg <dir> [-H <pattern>] [-t <type>] [--topic <topic>] [--budget <lines>]` prints matching sections, newest files first, then shorter sections first. The default budget is 500 content lines. The first matching section is emitted even if it exceeds the budget; subsequent sections stop at the first one that would exceed it.
- `mdd map <dir>` summarizes recent files, directory structure, topics, and content features. Recent means modified in the last three days.
- `mdd stats <dir>` reports file and line counts, frontmatter coverage, types, content features, and heading depth.

These directory commands, directory `toc`, and directory `fm` use ignore-aware traversal, follow symlinks, and exclude files named `README.md`, `AGENTS.md`, `CLAUDE.md`, and `CLAUDE.local.md` unless `--all-files` is given. Naming such a file directly (`mdd toc README.md`) always reads it.

## Links and frontmatter

Updated: 2026-11-02

- `mdd links <path> [--check | --broken]` lists the inline, reference, collapsed, and shortcut links in a file's Markdown body, with the destination alone as the target (no title or angle brackets). Images, autolinks (`<https://...>`), and links inside code or frontmatter are not reported (see issue #11). `--check` tests whether local target paths exist; `--broken` implies checking and prints only broken local targets. HTTP, HTTPS, and mail links are labeled external, not fetched. Anchor fragments are not validated, and plain `links` exits 0 even when targets are broken, while `--check` and `--broken` exit 1 when at least one local target is broken. Directory scans use the exclusions above.
- `mdd parts <file> [--filter key=value]` lists entries in a multi-frontmatter document. The filter compares the selected frontmatter value case-insensitively.
- `mdd fm <path>` prints frontmatter fields for a file or coverage for a directory. `--list-values <field>` lists unique values. `mdd` is read-only and never writes files. Directory `fm` scans use the same filename exclusions as the other directory commands.
